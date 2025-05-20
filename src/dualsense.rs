#![allow(dead_code)]

use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;

pub const SUBSYSTEM: &str = "hid";
pub const DRIVER: &str = "playstation";
pub const VENDOR_ID: u16 = 0x054C; // Sony
pub const PRODUCT_ID: u16 = 0x0CE6; // DualSense

/// Checks the 'sysname' for the correct vendor and product ID.
pub fn check_sysname(name: &OsStr) -> bool {
    let bytes = name.as_bytes();
    bytes.len() == 19 && &bytes[5..14] == "054C:0CE6".as_bytes()
}
