#![crate_type="staticlib"]
#![no_std]
#![feature(lang_items)]
//extern crate alloc;
extern crate core;
extern crate libc;
//extern crate std;

use core::prelude::*;
use core::fmt::write;
//use alloc::boxed;
use core::fmt;

macro_rules! write {
    ($dst:expr, $($arg:tt)*) => ((&mut *$dst).write_fmt(format_args!($($arg)*)))
}

macro_rules! writeln {
    ($dst:expr, $fmt:expr $($arg:tt)*) => (
        write!($dst, concat!($fmt, "\n") $($arg)*)
    )
}


pub fn myprint_args(fmt: fmt::Arguments) -> Result<(), core::fmt::Error>  {
  let kio = &mut ::Kio { lol: 1337 };
  let io = kio as &mut core::fmt::Writer;
  write!(io, "{}", fmt)
}

pub fn myprintln_args(fmt: fmt::Arguments) -> Result<(), core::fmt::Error>  {
  let kio = &mut ::Kio { lol: 1337 };
  let io = kio as &mut core::fmt::Writer;
  writeln!(io, "{}", fmt)
}

macro_rules! newprint {
    ($($arg:tt)*) => (myprint_args(format_args!($($arg)*)))
}

macro_rules! newprintln {
    ($($arg:tt)*) => (myprintln_args(format_args!($($arg)*)))
}

struct Kio {
  lol: int,
}

impl core::fmt::Writer for Kio {
  fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
    let sl = s.as_bytes();
    let len = s.len();

    unsafe {
      // FIXME: s is a Rust string here, but we need a C string
      rust_writek(sl, len);
    }
    Ok(()) // yes, we're lying
  }
}


extern {
  fn rust_writek(txt : &[u8], len: uint) -> ();
}

// std-module-trick to have the compiler emit correct expansions of `format_args!`
mod std { pub use core::fmt; }
macro_rules! print {
    ($($arg:tt)*) => (
      write(&mut Kio { lol: 1337 }, format_args!($($arg)*));
    )
}
/*let mut w = Vec::new();
write!(&mut w, "test");*/


// fix allocations
mod myheap;

extern {
  fn abort() -> !;
}

struct State {
    number: int,
    string: &'static str
}

impl fmt::Show for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // The `f` value implements the `Writer` trait, which is what the
        // write! macro is expecting. Note that this formatting ignores the
        // various flags provided to format strings.
        write!(f, "(num:{}, s:{})", self.number, self.string)
    }
}

static mut state : Option<State> = None;

extern {
  fn rust_printk(txt : &str) -> ();
}



const OS_DEFAULT_STACK_ESTIMATE: uint = 2 * (1 << 20);
#[no_mangle]
pub fn virtio_init() {

  // apparently, anything modifying global mutable state is considered unsafe...
  unsafe { state = Some(State { number: 1337, string: "" }); }

  // okay, even reading requires it
  unsafe {
    match state {
      Some(ref mut s) => s.number = 1338,
      None => (),
    }
  }

  unsafe { newprintln!("the state is now {}, lol", state); }
}


//use core::mem;

#[lang = "owned_box"]
pub struct Box<T>(*mut T);


#[start]
#[no_mangle]
pub unsafe fn hello_main() -> uint {
  let x = box 1i;

  let v = *x;
  if v == 2i {
    abort();
  }
  return 1337
}
/*
#[no_mangle]
pub extern fn dot_product(a: *const u32, a_len: u32,
                          b: *const u32, b_len: u32) -> u32 {
    use core::raw::Slice;

    // Convert the provided arrays into Rust slices.
    // The core::raw module guarantees that the Slice
    // structure has the same memory layout as a &[T]
    // slice.
    //
    // This is an unsafe operation because the compiler
    // cannot tell the pointers are valid.
    let (a_slice, b_slice): (&[u32], &[u32]) = unsafe {
        mem::transmute((
            Slice { data: a, len: a_len as uint },
            Slice { data: b, len: b_len as uint },
        ))
    };

    // Iterate over the slices, collecting the result
    let mut ret = 0;
    for (i, j) in a_slice.iter().zip(b_slice.iter()) {
        ret += (*i) * (*j);
    }
    return ret;
}*/


#[lang = "stack_exhausted"] extern fn stack_exhausted() {}
#[lang = "eh_personality"] extern fn eh_personality() {}
#[lang = "panic_fmt"] fn panic_fmt() -> ! { loop {} }
