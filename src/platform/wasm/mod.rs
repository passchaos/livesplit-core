#![allow(missing_docs)]

// Parses Date Times by ignoring them
// Doesn't add Durations to Date Times
// Durations are represented by 64-bit floats
pub mod chrono;

mod time;

pub use self::time::*;

use std::mem;

#[no_mangle]
pub extern "C" fn fmod(a: f64, b: f64) -> f64 {
    a - (a / b).floor() * b
}

#[no_mangle]
pub extern "C" fn fmodf(a: f32, b: f32) -> f32 {
    fmod(a as _, b as _) as _
}

#[no_mangle]
pub extern "C" fn alloc(size: usize) -> *mut u8 {
    let mut buf = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    mem::forget(buf);
    ptr
}

#[no_mangle]
pub extern "C" fn dealloc(ptr: *mut u8, cap: usize) {
    unsafe {
        let _buf = Vec::from_raw_parts(ptr, 0, cap);
    }
}
