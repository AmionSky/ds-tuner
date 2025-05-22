mod device;
mod dualsense;
mod bpf {
    include!(concat!(env!("OUT_DIR"), "/dualsense.skel.rs"));
}
mod input;

use anyhow::Result;
use bpf::DualsenseSkelBuilder;
use libbpf_rs::skel::{OpenSkel, SkelBuilder};
use libbpf_rs::{Link, MapCore, MapFlags};
use std::collections::HashMap;
use std::mem::MaybeUninit;
use std::path::{Path, PathBuf};

fn main() -> Result<()> {
    println!("DSMOD Started!");

    let event_rx = device::monitor_and_query()?;

    let mut bpfs: HashMap<PathBuf, Link> = HashMap::new();

    loop {
        let event = event_rx.recv().unwrap();
        match event {
            device::Event::Added(syspath) => {
                #[allow(clippy::map_entry)] // wtf clippy
                if !bpfs.contains_key(&syspath) {
                    println!("DualSense controller found at '{}'", syspath.display());
                    match load_bpf(&syspath) {
                        Ok(link) => {
                            println!("Loaded eBPF program for '{}'", syspath.display());
                            bpfs.insert(syspath, link);
                        }
                        Err(error) => eprintln!(
                            "Failed to load eBPF program for '{}' ({})",
                            syspath.display(),
                            error
                        ),
                    };
                } else {
                    eprintln!("Duplicate device found '{}'", syspath.display());
                }
            }
            device::Event::Removed(syspath) => {
                if bpfs.contains_key(&syspath) {
                    println!("DualSense controller removed from '{}'", syspath.display());
                    if bpfs.remove(&syspath).is_some() {
                        println!("Removed eBPF program for '{}'", syspath.display());
                    }
                }
            }
        }
    }
}

fn load_bpf(syspath: &Path) -> Result<Link> {
    let builder = DualsenseSkelBuilder::default();
    let mut open_object = MaybeUninit::uninit();
    let mut open_skel = builder.open(&mut open_object)?;

    insert_sysnum(&mut open_skel, syspath)?;

    let mut skel = open_skel.load()?;

    update_stick_lut(skel.maps.left_stick, &input::gen_stick_lut(0.08))?;
    update_stick_lut(skel.maps.right_stick, &input::gen_stick_lut(0.12))?;

    Ok(skel.maps.dsmod.attach_struct_ops()?)
}

fn insert_sysnum(open_skel: &mut bpf::OpenDualsenseSkel, syspath: &Path) -> Result<()> {
    let initval = open_skel
        .maps
        .dsmod
        .initial_value_mut()
        .ok_or(anyhow::anyhow!("Couldn't modify eBPF initial value!"))?;

    let sysnum = sysnum(syspath).ok_or(anyhow::anyhow!("Failed to get the device's sysnum!"))?;
    initval[0..4].copy_from_slice(&sysnum.to_le_bytes());

    Ok(())
}

// libudev's sysnum is broken
pub fn sysnum(syspath: &Path) -> Option<u32> {
    let path_str = syspath.to_str()?;
    let start = path_str
        .char_indices()
        .rev()
        .take_while(|(_, c)| c.is_ascii_hexdigit())
        .last()
        .map(|(i, _)| i)?;
    u32::from_str_radix(&path_str[start..], 16).ok()
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
