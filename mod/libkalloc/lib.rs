// The compiler needs to be instructed that this crate is an allocator in order
// to realize that when this is linked in another allocator like jemalloc should
// not be linked in
#![feature(allocator)]
#![allocator]

// Allocators are not allowed to depend on the standard library which in turn
// requires an allocator in order to avoid circular dependencies. This crate,
// however, can use all of libcore.
#![no_std]

// Let's give a unique name to our custom allocator
#![crate_name = "kalloc"]
#![crate_type = "rlib"]

// Our system allocator will use the in-tree libc crate for FFI bindings. Note
// that currently the external (crates.io) libc cannot be used because it links
// to the standard library (e.g. `#![no_std]` isn't stable yet), so that's why
// this specifically requires the in-tree version.
#![feature(libc)]
extern crate libc;

// Listed below are the five allocation functions currently required by custom
// allocators. Their signatures and symbol names are not currently typechecked
// by the compiler, but this is a future extension and are required to match
// what is found below.
//
// Note that the standard `malloc` and `realloc` functions do not provide a way
// to communicate alignment so this implementation would need to be improved
// with respect to alignment in that aspect.

#[no_mangle]
pub extern fn __rust_allocate(size: usize, _align: usize) -> *mut u8 {
    unsafe { libc::malloc(size as libc::size_t) as *mut u8 }
}

#[no_mangle]
pub extern fn __rust_deallocate(ptr: *mut u8, _old_size: usize, _align: usize) {
    unsafe { libc::free(ptr as *mut libc::c_void) }
}

#[no_mangle]
pub extern fn __rust_reallocate(ptr: *mut u8, _old_size: usize, size: usize,
                                _align: usize) -> *mut u8 {
    unsafe {
        libc::realloc(ptr as *mut libc::c_void, size as libc::size_t) as *mut u8
    }
}

#[no_mangle]
pub extern fn __rust_reallocate_inplace(_ptr: *mut u8, old_size: usize,
                                        _size: usize, _align: usize) -> usize {
    old_size // this api is not supported by libc
}

#[no_mangle]
pub extern fn __rust_usable_size(size: usize, _align: usize) -> usize {
    size
}
