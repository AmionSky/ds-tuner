use crate::Event;
use anyhow::Result;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::sync::mpsc::SyncSender;
use udev::mio::{Events, Interest, Poll, Token};

const SUBSYSTEM: &str = "hid";
const SUPPORTED: [(&str, &str); 1] = [
    ("054C", "0CE6"), // DualSense
];

pub fn monitor_and_query(tx: SyncSender<Event>) -> Result<()> {
    spawn_monitor(tx.clone());
    query(tx)?;
    Ok(())
}

/// Checks the 'sysname' for the correct vendor and product ID.
pub fn check_sysname(name: &OsStr) -> bool {
    let bytes = name.as_bytes();

    if bytes.len() != 19 {
        log::debug!("Device's SYSNAME length is invalid");
        return false;
    }

    let vendor = &bytes[5..9];
    let product = &bytes[10..14];

    for (vid, pid) in SUPPORTED {
        if vendor == vid.as_bytes() && product == pid.as_bytes() {
            return true;
        }
    }

    false
}

fn query(tx: SyncSender<Event>) -> Result<()> {
    let mut query = udev::Enumerator::new()?;
    query.match_subsystem(SUBSYSTEM)?;

    let list = query.scan_devices()?;
    for device in list {
        if check_sysname(device.sysname()) {
            let sysname = to_str(device.sysname());
            log::debug!("Found device: {sysname}");
            tx.send(Event::DeviceAdded(sysname))?;
        }
    }

    Ok(())
}

fn spawn_monitor(tx: SyncSender<Event>) {
    std::thread::Builder::new()
        .name("device_monitor".into())
        .spawn(move || {
            if let Err(error) = monitor(tx) {
                log::error!("Device monitor stopped: {error}");
            }
        })
        .expect("Failed to spawn device monitor thread!");
}

fn monitor(tx: SyncSender<Event>) -> std::io::Result<()> {
    let mut socket = udev::MonitorBuilder::new()?
        .match_subsystem(SUBSYSTEM)?
        .listen()?;

    let mut poll = Poll::new()?;
    poll.registry()
        .register(&mut socket, Token(0), Interest::READABLE)?;

    let mut events = Events::with_capacity(1);
    loop {
        poll.poll(&mut events, None)?;

        // Since all the events are in `socket` just ignore `events`
        for event in socket.iter().filter(|e| check_sysname(e.sysname())) {
            log::debug!(
                "Device event: Type={} Name={}",
                event.event_type(),
                event.sysname().display()
            );

            match event.event_type() {
                udev::EventType::Add => {
                    tx.send(Event::DeviceAdded(to_str(event.sysname())))
                        .expect("Failed to send device event!");
                }
                udev::EventType::Remove => {
                    tx.send(Event::DeviceRemoved(to_str(event.sysname())))
                        .expect("Failed to send device event!");
                }
                _ => (), // Ignore
            }
        }
    }
}

fn to_str(osstr: &OsStr) -> String {
    osstr.to_str().expect("Invalid UTF-8").to_string()
}
