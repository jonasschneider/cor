#![crate_type="staticlib"]
#![no_std]
#![feature(lang_items,unsafe_destructor,asm)]

extern crate core;

use core::prelude::*;

// Set up the `print!` and `println!` macros, printing to the kernel console
#[macro_use]
mod print;
mod std { pub use core::fmt; } // std-module-trick to fix expansion of `format_args!`

// Provide the compiler with implementations for heap data structures via kalloc
mod myheap;
mod boxed;

// cpuio library
mod cpuio;

// import submodules
mod block {
  pub mod virtio;
}




/*let mut w = Vec::new();
write!(&mut w, "test");*/

static mut state : Option<State> = None;


extern {
  fn abort() -> !;
}

struct State {
    number: int,
    string: &'static str
}

impl core::fmt::Show for State {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        // The `f` value implements the `Writer` trait, which is what the
        // write! macro is expecting. Note that this formatting ignores the
        // various flags provided to format strings.
        write!(f, "(num:{}, s:{})", self.number, self.string)
    }
}


#[no_mangle]
pub fn virtio_init(ioport : u16) {

  // apparently, anything modifying global mutable state is considered unsafe...
  unsafe { state = Some(State { number: 1337, string: "" }); }

  // okay, even reading requires it
  unsafe {
    match state {
      Some(ref mut s) => s.number = 1338,
      None => (),
    }
  }

  unsafe { println!("the state is now {}, lol", state); }
  let a_device = block::virtio::init(&ioport);
  println!("my device is: {}", a_device);
}

#[lang = "stack_exhausted"] extern fn stack_exhausted() {}
#[lang = "eh_personality"] extern fn eh_personality() {}
#[lang = "panic_fmt"] fn panic_fmt() -> ! { loop {} }
