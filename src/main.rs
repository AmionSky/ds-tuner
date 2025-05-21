mod device;
mod dualsense;
mod ffi;
mod bpf {
    include!(concat!(env!("OUT_DIR"), "/dualsense.skel.rs"));
}
mod input;

use anyhow::Result;
use bpf::DualsenseSkelBuilder;
use ffi::*;
use libbpf_rs::skel::{OpenSkel, SkelBuilder};
use libbpf_rs::{Link, MapCore, MapFlags, ProgramInput};
use std::collections::HashMap;
use std::mem::MaybeUninit;
use std::path::{Path, PathBuf};

pub fn dev_id(device: &udev::Device) -> u32 {
    let hid_sys = String::from(device.sysname().to_str().unwrap());
    u32::from_str_radix(&hid_sys[15..], 16).unwrap()
}

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
                    println!("DualSense Controller connected @ {}", syspath.display());
                    match load_bpf(&syspath) {
                        Ok(link) => {
                            bpfs.insert(syspath, link);
                            println!("eBPF successfully loaded for device!");
                        }
                        Err(error) => println!("Failed to load eBPF: {error}"),
                    };
                }
            }
            device::Event::Removed(syspath) => {
                if bpfs.contains_key(&syspath) {
                    println!("DualSense Controller disconnected @ {}", syspath.display());
                    bpfs.remove(&syspath);
                    println!("eBPF successfully removed!");
                }
            }
        }
    }
}

fn load_bpf(syspath: &Path) -> Result<Link> {
    let device = udev::Device::from_syspath(syspath)?;
    let inum = dev_id(&device);

    let builder = DualsenseSkelBuilder::default();
    let mut open_object = MaybeUninit::uninit();
    let mut open_skel = builder.open(&mut open_object)?;

    {
        let initval = open_skel
            .maps
            .edit_values
            .initial_value_mut()
            .ok_or(anyhow::anyhow!("Could not set 'initval'!"))?;
        initval[0..4].copy_from_slice(&inum.to_le_bytes());
    }

    let mut skel = open_skel.load()?;

    // Setup
    {
        let lut = input::gen_input();

        // Insert data
        for (k, v) in lut.into_iter().enumerate() {
            let key = (k as u32).to_ne_bytes();
            let val = v.to_ne_bytes();
            skel.maps.left_stick.update(&key, &val, MapFlags::ANY)?;
            skel.maps.right_stick.update(&key, &val, MapFlags::ANY)?;
        }

        let mut cfg = edit_config { dummy: 0 };
        let mut input = ProgramInput::default();
        unsafe { input.context_in = Some(cfg.as_slice_mut()) };
        skel.progs.setup.test_run(input)?;
    }

    Ok(skel.maps.edit_values.attach_struct_ops()?)
}
