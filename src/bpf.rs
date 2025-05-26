mod skel {
    include!(concat!(env!("OUT_DIR"), "/dualsense.skel.rs"));
}

use crate::conf::Config;
use crate::input::gen_stick_lut;
use anyhow::{Result, anyhow};
use libbpf_rs::skel::{OpenSkel, SkelBuilder};
use libbpf_rs::{Link, MapCore, MapFlags};
use std::mem::MaybeUninit;

pub fn load(sysname: &str, config: &Config) -> Result<Link> {
    let builder = skel::DualsenseSkelBuilder::default();
    let mut open_object = MaybeUninit::uninit();
    let mut open_skel = builder.open(&mut open_object)?;

    insert_sysnum(&mut open_skel, sysname)?;

    let mut skel = open_skel.load()?;

    update_stick_lut(skel.maps.left_stick, &gen_stick_lut(&config.stick.left))?;
    update_stick_lut(skel.maps.right_stick, &gen_stick_lut(&config.stick.right))?;

    Ok(skel.maps.dsmod.attach_struct_ops()?)
}

fn insert_sysnum(open_skel: &mut skel::OpenDualsenseSkel, sysname: &str) -> Result<()> {
    let initval = open_skel
        .maps
        .dsmod
        .initial_value_mut()
        .ok_or(anyhow!("Couldn't modify eBPF initial value!"))?;

    let sysnum = sysnum(sysname).ok_or(anyhow!("Failed to get the device's sysnum!"))?;
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
