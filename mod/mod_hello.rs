#![crate_type="staticlib"]
#![no_std]
#![feature(globs,lang_items)]
#![allow(unused_attribute)]

extern crate core;

use core::prelude::*;
use core::panicking::panic;


#[link(name = "cor")]
extern {
  fn cor_hitmarker() -> ();
}

#[start]
#[no_mangle]
pub unsafe fn cmod_main() {

  let x = 5i;

  if x % 2 == 1i {
    cor_hitmarker();
  }
}

#[lang = "stack_exhausted"] extern fn stack_exhausted() {}
#[lang = "eh_personality"] extern fn eh_personality() {}
#[lang = "panic_fmt"] fn panic_fmt() -> ! { loop {} }
#[lang = "fail_fmt"] fn fail_fmt() -> ! { loop {} }
