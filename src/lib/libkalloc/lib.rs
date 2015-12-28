#![feature(allocator)]
#![allocator]
#![no_std]

#![crate_name = "kalloc"]
#![crate_type = "rlib"]

use core::ptr::copy;

extern "C" {
  fn rust_allocate(size: usize, align: usize) -> *mut u8;
  fn rust_deallocate(mem: *mut u8, size: usize, align: usize);
}

#[no_mangle]
pub unsafe extern fn __rust_allocate(size: usize, _align: usize) -> *mut u8 {
    rust_allocate(size as usize, _align as usize) as *mut u8
}

#[no_mangle]
pub unsafe extern fn __rust_deallocate(ptr: *mut u8, _old_size: usize, _align: usize) {
    rust_deallocate(ptr as *mut u8, _old_size as usize, _align as usize)
}

#[no_mangle]
pub unsafe extern fn __rust_reallocate(ptr: *mut u8, _old_size: usize, size: usize,
                                _align: usize) -> *mut u8 {
    // yep, this is how we roll
    let new = rust_allocate(size as usize, _align as usize);
    copy(ptr, new as *mut u8, _old_size);
    new as *mut u8
}

#[no_mangle]
pub extern fn __rust_reallocate_inplace(_ptr: *mut u8, old_size: usize,
                                        _size: usize, _align: usize) -> usize {
    old_size // this api is not supported by us
}

#[no_mangle]
pub extern fn __rust_usable_size(size: usize, _align: usize) -> usize {
    size
}
