//! Rebuild `monos.wasm` with:
//!
//! ```
//! rustc +nightly -g --target wasm32-unknown-unknown monos.rs -o monos.wasm -C lto=fat -C opt-level=z
//! ```

#![cfg(target_arch = "wasm32")]
#![crate_type = "cdylib"]

use std::slice;

extern "C" {
    fn hello(f: extern "C" fn() -> u32);
}

trait Code {
    extern "C" fn code() -> u32;
}

#[inline(never)]
fn generic<C: Code>() {
    unsafe {
        hello(C::code);
    }
}

struct Zero;
impl Code for Zero {
    extern "C" fn code() -> u32 {
        0
    }
}

struct One;
impl Code for One {
    extern "C" fn code() -> u32 {
        1
    }
}

struct Two;
impl Code for Two {
    extern "C" fn code() -> u32 {
        2
    }
}

#[no_mangle]
pub extern "C" fn trigger_generic_monos() {
    generic::<Zero>();
    generic::<One>();
    generic::<Two>();
}

#[no_mangle]
pub extern "C" fn sort_u32s(ptr: *mut u32, len: usize) {
    unsafe {
        let slice = slice::from_raw_parts_mut(ptr, len);
        slice.sort();
    }
}

#[no_mangle]
pub extern "C" fn sort_u8s(ptr: *mut u8, len: usize) {
    unsafe {
        let slice = slice::from_raw_parts_mut(ptr, len);
        slice.sort();
    }
}

#[no_mangle]
pub extern "C" fn sort_i32s(ptr: *mut i32, len: usize) {
    unsafe {
        let slice = slice::from_raw_parts_mut(ptr, len);
        slice.sort();
    }
}

#[no_mangle]
pub extern "C" fn push_and_sort_u32s(ptr: *mut u32, cap: usize, len: usize) {
    unsafe {
        let mut vec = Vec::from_raw_parts(ptr, len, cap);
        vec.push(0);
        vec.push(1);
        vec.push(2);
        vec.sort();
    }
}

#[no_mangle]
pub extern "C" fn push_and_sort_u8s(ptr: *mut u8, cap: usize, len: usize) {
    unsafe {
        let mut vec = Vec::from_raw_parts(ptr, len, cap);
        vec.push(0);
        vec.push(1);
        vec.push(2);
        vec.sort();
    }
}

#[no_mangle]
pub extern "C" fn push_and_sort_i32s(ptr: *mut i32, cap: usize, len: usize) {
    unsafe {
        let mut vec = Vec::from_raw_parts(ptr, len, cap);
        vec.push(0);
        vec.push(1);
        vec.push(2);
        vec.sort();
    }
}
