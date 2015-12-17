#![feature(allocator)]
#![allocator]
#![no_std]

#![crate_name = "kalloc"]
#![crate_type = "rlib"]

// use in-tree libc, not crates.io
#![feature(libc)]
extern crate libc;

extern "C" {
  fn rust_allocate(size: libc::size_t, align: libc::size_t) -> *mut libc::c_void;
  fn rust_deallocate(mem: *mut libc::c_void, size: libc::size_t, align: libc::size_t);
}

#[no_mangle]
pub extern fn __rust_allocate(size: usize, _align: usize) -> *mut u8 {
    unsafe { rust_allocate(size as libc::size_t, _align as libc::size_t) as *mut u8 }
}

#[no_mangle]
pub extern fn __rust_deallocate(ptr: *mut u8, _old_size: usize, _align: usize) {
    unsafe { rust_deallocate(ptr as *mut libc::c_void, _old_size as libc::size_t, _align as libc::size_t) }
}

#[no_mangle]
pub extern fn __rust_reallocate(ptr: *mut u8, _old_size: usize, size: usize,
                                _align: usize) -> *mut u8 {
    panic!("realloc()!");
    // unsafe {
    //     libc::realloc(ptr as *mut libc::c_void, size as libc::size_t) as *mut u8
    // }
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
