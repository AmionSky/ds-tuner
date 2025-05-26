mod bpf;
mod conf;
mod device;
mod dualsense;
mod input;

use anyhow::Result;
use conf::{Config, ConfigWatcher};
use libbpf_rs::Link;
use std::collections::HashMap;

fn main() {
    init_logger().expect("Failed to initialize logger!");

    if let Err(error) = start() {
        log::error!("Fatal error: {error}");
        panic!("{error}");
    }
}

#[derive(Debug)]
enum Event {
    DeviceAdded(String),
    DeviceRemoved(String),
    ConfigChanged,
}

struct BpfStore(HashMap<String, Link>);

impl BpfStore {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn contains(&self, sysname: &String) -> bool {
        self.0.contains_key(sysname)
    }

    pub fn keys(&self) -> Vec<String> {
        self.0.keys().map(|k| k.to_owned()).collect()
    }

    pub fn load(&mut self, sysname: String, config: &Config) {
        match bpf::load(&sysname, config) {
            Ok(link) => {
                log::debug!("Loaded eBPF program for {sysname}");
                self.0.insert(sysname, link);
            }
            Err(error) => {
                log::error!("Failed to load eBPF program for {sysname} ({error})");
            }
        };
    }

    pub fn unload(&mut self, sysname: &String) {
        if self.0.remove(sysname).is_some() {
            log::debug!("Removed eBPF program for {sysname}");
        }
    }
}

fn start() -> Result<()> {
    log::info!("DSMOD v{} started!", env!("CARGO_PKG_VERSION"));

    let (main_tx, main_rx) = std::sync::mpsc::sync_channel(1);

    let config = ConfigWatcher::init(main_tx.clone());
    let mut bpf_store = BpfStore::new();
    device::monitor_and_query(main_tx.clone())?;

    loop {
        match main_rx.recv()? {
            Event::DeviceAdded(sysname) => {
                if !bpf_store.contains(&sysname) {
                    log::info!("DualSense controller connected: {sysname}");
                    bpf_store.load(sysname, &config.config());
                } else {
                    // Probably can only be caused by a race condition between
                    // the start of the device monitor and the manual query
                    log::warn!("Duplicate device found: {sysname}");
                }
            }
            Event::DeviceRemoved(sysname) => {
                if bpf_store.contains(&sysname) {
                    log::info!("DualSense controller disconnected: {sysname}");
                    bpf_store.unload(&sysname);
                }
            }
            Event::ConfigChanged => {
                log::info!("Configuration changed. Reloading.");
                for sysname in bpf_store.keys() {
                    bpf_store.unload(&sysname);
                    bpf_store.load(sysname, &config.config());
                }
            }
        }
    }
}

#[cfg(not(feature = "systemd"))]
fn init_logger() -> Result<()> {
    use simplelog::{ColorChoice, ConfigBuilder, LevelFilter, TermLogger, TerminalMode};

    TermLogger::init(
        LevelFilter::Debug,
        ConfigBuilder::new()
            .set_thread_level(LevelFilter::Trace)
            .set_target_level(LevelFilter::Trace)
            .build(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )?;

    Ok(())
}

#[cfg(feature = "systemd")]
fn init_logger() -> Result<()> {
    use systemd_journal_logger::JournalLog;

    JournalLog::new()?.install()?;
    log::set_max_level(log::LevelFilter::Info);

    Ok(())
}
