use libc::{LOCK_EX, LOCK_NB, flock};
use std::fs::File;
use std::io::{Error, ErrorKind};
use std::os::fd::AsRawFd;
use std::path::Path;

pub struct SingleInstance {
    lock: Option<File>,
}

impl SingleInstance {
    pub fn new() -> Self {
        let path = Path::new("/tmp/ds-tuner.lock");

        match File::create(path) {
            Ok(file) => {
                if unsafe { flock(file.as_raw_fd(), LOCK_EX | LOCK_NB) } == 0 {
                    return Self { lock: Some(file) };
                } else {
                    let error = Error::last_os_error();
                    if error.kind() != ErrorKind::WouldBlock {
                        log::error!("Failed to lock lockfile ({error})");
                    }
                }
            }
            Err(error) => log::error!("Failed to create lockfile ({error})"),
        }

        Self { lock: None }
    }

    pub fn single(&self) -> bool {
        self.lock.is_some()
    }
}
