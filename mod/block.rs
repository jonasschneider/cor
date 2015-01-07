#![crate_type="staticlib"]
#![no_std]
#![feature(globs,lang_items,macro_rules)]
//extern crate alloc;
extern crate core;
extern crate libc;
//extern crate std;

use core::prelude::*;
use core::fmt::write;
//use alloc::boxed;
use core::fmt::Arguments;


macro_rules! mywrite {
    ($dst:expr, $($arg:tt)*) => ((&mut *$dst).write_fmt(format_args!($($arg)*)))
}

macro_rules! mywriteln {
    ($dst:expr, $fmt:expr $($arg:tt)*) => (
        mywrite!($dst, concat!($fmt, "\n") $($arg)*)
    )
}


pub fn myprintln_args(fmt: Arguments) -> Result<(), core::fmt::Error>  {
  let kio = &mut ::Kio { lol: 1337 };
  let io = kio as &mut core::fmt::Writer;
  mywriteln!(io, "{}", fmt)
}

macro_rules! newprint {
    ($($arg:tt)*) => (myprintln_args(format_args!($($arg)*)))
}

struct Kio {
  lol: int,
}

impl core::fmt::Writer for Kio {
  fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
    unsafe {
      // FIXME: s is a Rust string here, but we need a C string
      rust_printk(s);
    }
    Ok(()) // yes, we're lying
  }
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

static mut state : Option<State> = None;

extern {
  fn rust_printk(txt : &str) -> ();
}

const OS_DEFAULT_STACK_ESTIMATE: uint = 2 * (1 << 20);
#[no_mangle]
pub fn virtio_init() {

/*  unsafe { rust_printk("hai from rust\n"); }


  // apparently, anything modifying global mutable state is considered unsafe...
  unsafe { state = Some(State { number: 1337, string: "" }); }

  // okay, even reading requires it
  unsafe {
    match state {
      Some(ref mut s) => s.number = 1338,
      None => (),
    }
  }
*/
  let num = 3;

  newprint!("the number is {} lol\n", num);
/*
  let args = format_args!("now the state is");
  unsafe {
    write(1, args);
  }*/
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
