mod bpf;
mod conf;
mod device;
mod dualsense;
mod input;

use self::device::Event;
use anyhow::{Result, anyhow};
use libbpf_rs::Link;
use std::collections::HashMap;

fn main() -> Result<()> {
    init_logger()?;

    log::info!("DSMOD v{} started!", env!("CARGO_PKG_VERSION"));

    let config = conf::load().map_err(|e| anyhow!("Failed to load configuration file! ({e})"))?;

    if !config.enabled {
        log::info!("DSMOD is disabled in config!");
        return Ok(());
    }

    let event_rx = device::monitor_and_query()?;
    let mut bpfs: HashMap<String, Link> = HashMap::new();

    loop {
        match event_rx.recv()? {
            Event::Added(sysname) => {
                #[allow(clippy::map_entry)] // wtf clippy
                if !bpfs.contains_key(&sysname) {
                    log::info!("DualSense controller connected: {sysname}");
                    match bpf::load(&sysname, &config) {
                        Ok(link) => {
                            log::debug!("Loaded eBPF program for {sysname}");
                            bpfs.insert(sysname, link);
                        }
                        Err(error) => {
                            log::error!("Failed to load eBPF program for {sysname} ({error})");
                        }
                    };
                } else {
                    // Probably can only be caused by a race condition between
                    // the start of the device monitor and the manual query
                    log::warn!("Duplicate device found: {sysname}");
                }
            }
            Event::Removed(sysname) => {
                if bpfs.contains_key(&sysname) {
                    log::info!("DualSense controller disconnected: {sysname}");
                    if bpfs.remove(&sysname).is_some() {
                        log::debug!("Removed eBPF program for {sysname}");
                    }
                }
            }
        }
    }
}

fn init_logger() -> Result<()> {
    use simplelog::{ColorChoice, Config, LevelFilter, TermLogger, TerminalMode};
    use systemd_journal_logger::{JournalLog, connected_to_journal};

    if connected_to_journal() {
        JournalLog::new()?.install()?;
        log::set_max_level(LevelFilter::Info);
    } else {
        TermLogger::init(
            LevelFilter::Debug,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        )?;
    }

    Ok(())
}
