mod device;
mod dualsense;
mod input;
mod bpf {
    include!(concat!(env!("OUT_DIR"), "/dualsense.skel.rs"));
}

use self::bpf::DualsenseSkelBuilder;
use self::device::Event;
use anyhow::Result;
use libbpf_rs::skel::{OpenSkel, SkelBuilder};
use libbpf_rs::{Link, MapCore, MapFlags};
use std::collections::HashMap;
use std::mem::MaybeUninit;

fn main() -> Result<()> {
    init_logger()?;

    log::info!("DSMOD v{} started!", env!("CARGO_PKG_VERSION"));

    let event_rx = device::monitor_and_query()?;
    let mut bpfs: HashMap<String, Link> = HashMap::new();

    loop {
        let event = event_rx.recv().unwrap();
        match event {
            Event::Added(sysname) => {
                #[allow(clippy::map_entry)] // wtf clippy
                if !bpfs.contains_key(&sysname) {
                    log::info!("DualSense controller connected: '{sysname}'");
                    match load_bpf(&sysname) {
                        Ok(link) => {
                            log::info!("Loaded eBPF program for '{sysname}'");
                            bpfs.insert(sysname, link);
                        }
                        Err(error) => {
                            log::error!("Failed to load eBPF program for '{sysname}' ({error})");
                        }
                    };
                } else {
                    log::warn!("Duplicate device found: '{sysname}'");
                }
            }
            Event::Removed(sysname) => {
                if bpfs.contains_key(&sysname) {
                    log::info!("DualSense controller disconnected: '{sysname}'");
                    if bpfs.remove(&sysname).is_some() {
                        log::info!("Removed eBPF program for '{sysname}'");
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

fn load_bpf(sysname: &str) -> Result<Link> {
    let builder = DualsenseSkelBuilder::default();
    let mut open_object = MaybeUninit::uninit();
    let mut open_skel = builder.open(&mut open_object)?;

    insert_sysnum(&mut open_skel, sysname)?;

    let mut skel = open_skel.load()?;

    update_stick_lut(skel.maps.left_stick, &input::gen_stick_lut(0.08))?;
    update_stick_lut(skel.maps.right_stick, &input::gen_stick_lut(0.12))?;

    Ok(skel.maps.dsmod.attach_struct_ops()?)
}

fn insert_sysnum(open_skel: &mut bpf::OpenDualsenseSkel, sysname: &str) -> Result<()> {
    let initval = open_skel
        .maps
        .dsmod
        .initial_value_mut()
        .ok_or(anyhow::anyhow!("Couldn't modify eBPF initial value!"))?;

    let sysnum = sysnum(sysname).ok_or(anyhow::anyhow!("Failed to get the device's sysnum!"))?;
    initval[0..4].copy_from_slice(&sysnum.to_le_bytes());

    Ok(())
}

// libudev's sysnum is broken (it uses decimal instead of hexadecimal)
pub fn sysnum(sysname: &str) -> Option<u32> {
    let start = sysname
        .char_indices()
        .rev()
        .take_while(|(_, c)| c.is_ascii_hexdigit())
        .last()
        .map(|(i, _)| i)?;
    u32::from_str_radix(&sysname[start..], 16).ok()
}

fn update_stick_lut<M: MapCore>(map: M, lut: &[u16]) -> libbpf_rs::Result<()> {
    debug_assert_eq!(lut.len(), 256 * 256);
    for (k, v) in lut.iter().enumerate() {
        let key = (k as u32).to_ne_bytes();
        let val = v.to_ne_bytes();
        map.update(&key, &val, MapFlags::ANY)?;
    }
    Ok(())
}
