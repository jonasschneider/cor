#![crate_type="staticlib"]
#![no_std]
#![feature(globs,lang_items)]

//extern crate alloc;
extern crate core;
extern crate libc;

//use alloc::boxed;
mod myheap;
//use core::prelude::*;

//use core::mem;

extern {
  fn abort() -> !;
}

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
