use crate::dualsense::{DRIVER, SUBSYSTEM, check_sysname};
use anyhow::Result;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread::spawn;
use udev::mio::{Events, Interest, Poll, Token};

#[derive(Debug)]
pub enum Event {
    Added(PathBuf),
    Removed(PathBuf),
}

pub fn monitor_and_query() -> Result<Receiver<Event>> {
    let (tx, rx) = channel();
    let query_tx = tx.clone();
    spawn(|| monitor(tx).expect("Udev monitor failed!"));
    query(query_tx)?;
    Ok(rx)
}

fn query(tx: Sender<Event>) -> Result<()> {
    let mut query = udev::Enumerator::new()?;
    query.match_subsystem(SUBSYSTEM)?;
    query.match_property("DRIVER", DRIVER)?;

    let list = query.scan_devices()?;
    for device in list {
        if check_sysname(device.sysname()) {
            tx.send(Event::Added(device.syspath().to_path_buf()))?;
        }
    }

    Ok(())
}

fn monitor(tx: Sender<Event>) -> std::io::Result<()> {
    let socket = udev::MonitorBuilder::new()?
        .match_subsystem(SUBSYSTEM)?
        .listen()?;

    poll(socket, tx)
}

fn poll(mut socket: udev::MonitorSocket, tx: Sender<Event>) -> std::io::Result<()> {
    let mut poll = Poll::new()?;
    poll.registry().register(
        &mut socket,
        Token(0),
        Interest::READABLE | Interest::WRITABLE,
    )?;

    let mut events = Events::with_capacity(8);
    loop {
        poll.poll(&mut events, None)?;

        for event in &events {
            if event.token() == Token(0) && event.is_writable() {
                socket
                    .iter()
                    .filter(|e| check_sysname(e.sysname()))
                    .for_each(|e| match e.event_type() {
                        udev::EventType::Bind => {
                            tx.send(Event::Added(e.syspath().to_path_buf()))
                                .expect("Failed to send event!");
                        }
                        udev::EventType::Unbind => {
                            tx.send(Event::Removed(e.syspath().to_path_buf()))
                                .expect("Failed to send event!");
                        }
                        _ => (), // Ignore
                    });
            }
        }
    }
}
