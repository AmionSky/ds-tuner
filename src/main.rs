mod ffi;
mod bpf {
    include!(concat!(env!("OUT_DIR"), "/dualsense.skel.rs"));
}

use anyhow::Result;
use bpf::DualsenseSkelBuilder;
use ffi::*;
use libbpf_rs::skel::{OpenSkel, SkelBuilder};
use libbpf_rs::{MapCore, ProgramInput};
use std::mem::MaybeUninit;
use std::path::Path;

pub fn dev_id(device: &udev::Device) -> u32 {
    let hid_sys = String::from(device.sysname().to_str().unwrap());
    u32::from_str_radix(&hid_sys[15..], 16).unwrap()
}

fn main() -> Result<()> {
    println!("Hello, world!");

    let dev_path = Path::new("/sys/bus/hid/devices/0005:054C:0CE6.000F");
    let dev = udev::Device::from_syspath(dev_path)?;

    println!("udev name: {}", dev.sysname().display());

    println!("udev properties:");
    for prop in dev.properties() {
        println!("  {:?} = {:?}", prop.name(), prop.value());
    }
    println!("udev attributes:");
    for attr in dev.attributes() {
        println!("  {:?} = {:?}", attr.name(), attr.value());
    }

    // unsafe {
    //     if libbpf_rs::libbpf_sys::libbpf_set_memlock_rlim(64) != 0 {
    //         panic!("Failed to set memlock_rlim");
    //     }
    // }

    let builder = DualsenseSkelBuilder::default();
    let mut open_object = MaybeUninit::uninit();
    let mut open_skel = builder.open(&mut open_object)?;

    {
        let initval = open_skel.maps.edit_values.initial_value_mut().unwrap();
        initval[0..4].copy_from_slice(&dev_id(&dev).to_le_bytes());
    }
    // open_skel.maps.edit_values.set_initial_value(&dev_id(&dev).to_le_bytes())?;

    println!("Init val: {:?}", open_skel.maps.edit_values.initial_value());
    // println!("Init val 2: {:?}", &open_skel.maps.rodata.initial_value().unwrap()[..]);

    let mut skel = open_skel.load()?;

    let rdesc = std::fs::read(dev_path.join("report_descriptor"))?;
    let rdesc_len = rdesc.len();
    let mut rdesc_data = [0; 4096];
    rdesc_data[..rdesc_len].copy_from_slice(&rdesc);

    let mut args = hid_bpf_probe_args {
        hid: dev_id(&dev),            //device.id(),
        rdesc_size: rdesc_len as u32, //length as u32,
        rdesc: rdesc_data,            //buffer.try_into().unwrap(),
        retval: -1,
    };

    println!("RETVAL: {}", args.retval);

    let mut input = ProgramInput::default();
    unsafe { input.context_in = Some(args.as_slice_mut()) };

    let output = skel.progs.probe.test_run(input)?;
    println!("probe output: {output:#?}");
    println!("RETVAL: {}", args.retval);
    // skel.progs.probe
    // skel.attach()?;

    // setup?
    let mut cfg = edit_config {
        ls_lt: [127; 256],
        rs_lt: [127; 256],
    };
    let mut input = ProgramInput::default();
    unsafe { input.context_in = Some(cfg.as_slice_mut()) };
    skel.progs.setup.test_run(input)?;

    // let attype = open_skel.progs.attach_prog.att

    println!(
        "Attach type of 'probe': {:?}",
        skel.progs.probe.attach_type()
    );
    println!(
        "Attach type of 'edit_values_event': {:?}",
        skel.progs.edit_values_event.attach_type()
    );

    println!(
        "Map type of 'edit_values': {:?}",
        skel.maps.edit_values.map_type()
    );

    let _link = skel.maps.edit_values.attach_struct_ops()?;
    // link.pin(path)

    // std::thread::sleep(std::time::Duration::from_secs(10));

    let (tx, rx) = std::sync::mpsc::channel();
    ctrlc::set_handler(move || tx.send(()).unwrap()).unwrap();
    rx.recv().unwrap();

    Ok(())
}
