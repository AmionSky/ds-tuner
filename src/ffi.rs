use std::mem::size_of;
use std::slice::{from_raw_parts, from_raw_parts_mut};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
#[allow(non_camel_case_types)]
pub struct edit_config {
    pub dummy: u8,
}

#[allow(dead_code)]
pub trait StructAsBytes {
    unsafe fn as_slice(&self) -> &[u8];
    unsafe fn as_slice_mut(&mut self) -> &mut [u8];
}

impl<T: Sized> StructAsBytes for T {
    unsafe fn as_slice(&self) -> &[u8] {
        unsafe { from_raw_parts((self as *const T) as *const u8, size_of::<T>()) }
    }

    unsafe fn as_slice_mut(&mut self) -> &mut [u8] {
        unsafe { from_raw_parts_mut((self as *mut T) as *mut u8, size_of::<T>()) }
    }
}
