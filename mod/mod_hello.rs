#![crate_type="rlib"]
#![no_std]
#![feature(globs, phase)]
#![allow(unused_attribute)]

extern crate core;

#[phase(plugin, link)]
extern crate syscall;

use core::prelude::*;
use core::{mem, raw, intrinsics};

fn exit(n: uint) -> ! {
    unsafe {
        syscall!(EXIT, n);
        intrinsics::unreachable()
    }
}

fn write(fd: uint, buf: &[u8]) {
    unsafe {
        syscall!(WRITE, fd, buf.as_ptr(), buf.len());
    }
}

#[no_mangle]
pub unsafe fn main() {
    exit(123);
    // Make a Rust value representing the string constant we stashed
    // in the ELF file header.
    let slice_repr: raw::Slice<u8> = raw::Slice {
        data: 0x00400008 as *const u8,
        len: 7,
    };
    let message: &'static [u8] = mem::transmute(slice_repr);

    write(1, message);
    exit(0);
}
