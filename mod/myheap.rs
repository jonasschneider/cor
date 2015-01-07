// TODO: do we still need this here somewhere?
// #[lang = "owned_box"]
// pub struct Box<T>(*mut T);


// Copyright 2014-2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use core::ptr::PtrExt;

extern {
  fn abort() -> !;
}

// FIXME: #13996: mark the `allocate` and `reallocate` return value as `noalias`

/// Return a pointer to `size` bytes of memory aligned to `align`.
///
/// On failure, return a null pointer.
///
/// Behavior is undefined if the requested size is 0 or the alignment is not a
/// power of 2. The alignment must be no larger than the largest supported page
/// size on the platform.
#[inline]
pub unsafe fn allocate(size: uint, align: uint) -> *mut u8 {
    imp::allocate(size, align)
}

/// Resize the allocation referenced by `ptr` to `size` bytes.
///
/// On failure, return a null pointer and leave the original allocation intact.
///
/// Behavior is undefined if the requested size is 0 or the alignment is not a
/// power of 2. The alignment must be no larger than the largest supported page
/// size on the platform.
///
/// The `old_size` and `align` parameters are the parameters that were used to
/// create the allocation referenced by `ptr`. The `old_size` parameter may be
/// any value in range_inclusive(requested_size, usable_size).
#[inline]
pub unsafe fn reallocate(ptr: *mut u8, old_size: uint, size: uint, align: uint) -> *mut u8 {
    imp::reallocate(ptr, old_size, size, align)
}

/// Resize the allocation referenced by `ptr` to `size` bytes.
///
/// If the operation succeeds, it returns `usable_size(size, align)` and if it
/// fails (or is a no-op) it returns `usable_size(old_size, align)`.
///
/// Behavior is undefined if the requested size is 0 or the alignment is not a
/// power of 2. The alignment must be no larger than the largest supported page
/// size on the platform.
///
/// The `old_size` and `align` parameters are the parameters that were used to
/// create the allocation referenced by `ptr`. The `old_size` parameter may be
/// any value in range_inclusive(requested_size, usable_size).
#[inline]
pub unsafe fn reallocate_inplace(ptr: *mut u8, old_size: uint, size: uint, align: uint) -> uint {
    imp::reallocate_inplace(ptr, old_size, size, align)
}

/// Deallocates the memory referenced by `ptr`.
///
/// The `ptr` parameter must not be null.
///
/// The `old_size` and `align` parameters are the parameters that were used to
/// create the allocation referenced by `ptr`. The `old_size` parameter may be
/// any value in range_inclusive(requested_size, usable_size).
#[inline]
pub unsafe fn deallocate(ptr: *mut u8, old_size: uint, align: uint) {
    imp::deallocate(ptr, old_size, align)
}

/// Returns the usable size of an allocation created with the specified the
/// `size` and `align`.
#[inline]
pub fn usable_size(size: uint, align: uint) -> uint {
    imp::usable_size(size, align)
}

/// Prints implementation-defined allocator statistics.
///
/// These statistics may be inconsistent if other threads use the allocator
/// during the call.
#[unstable]
pub fn stats_print() {
    imp::stats_print();
}

/// An arbitrary non-null address to represent zero-size allocations.
///
/// This preserves the non-null invariant for types like `Box<T>`. The address may overlap with
/// non-zero-size memory allocations.
pub const EMPTY: *mut () = 0x1 as *mut ();

/// The allocator for unique pointers.
#[cfg(not(test))]
#[lang="exchange_malloc"]
#[inline]
unsafe fn exchange_malloc(size: uint, align: uint) -> *mut u8 {
    if size == 0 {
        EMPTY as *mut u8
    } else {
        let ptr = allocate(size, align);
        if ptr.is_null() { oom() }
        ptr
    }
}

#[cfg(not(test))]
#[lang="exchange_free"]
#[inline]
unsafe fn exchange_free(ptr: *mut u8, old_size: uint, align: uint) {
    deallocate(ptr, old_size, align);
}

// The minimum alignment guaranteed by the architecture. This value is used to
// add fast paths for low alignment values. In practice, the alignment is a
// constant at the call site and the branch will be optimized out.
#[cfg(any(target_arch = "arm",
          target_arch = "mips",
          target_arch = "mipsel"))]
const MIN_ALIGN: uint = 8;
#[cfg(any(target_arch = "x86",
          target_arch = "x86_64",
          target_arch = "aarch64"))]
const MIN_ALIGN: uint = 16;

#[cfg(external_funcs)]
mod imp {
    extern {
        fn rust_allocate(size: uint, align: uint) -> *mut u8;
        fn rust_deallocate(ptr: *mut u8, old_size: uint, align: uint);
        fn rust_reallocate(ptr: *mut u8, old_size: uint, size: uint, align: uint) -> *mut u8;
        fn rust_reallocate_inplace(ptr: *mut u8, old_size: uint, size: uint,
                                   align: uint) -> uint;
        fn rust_usable_size(size: uint, align: uint) -> uint;
        fn rust_stats_print();
    }

    #[inline]
    pub unsafe fn allocate(size: uint, align: uint) -> *mut u8 {
        rust_allocate(size, align)
    }


    #[inline]
    pub unsafe fn reallocate(ptr: *mut u8, old_size: uint, size: uint,
                                     align: uint) -> *mut u8 {
        rust_reallocate(ptr, old_size, size, align)
    }

    #[inline]
    pub unsafe fn reallocate_inplace(ptr: *mut u8, old_size: uint, size: uint,
                                     align: uint) -> uint {
        rust_reallocate_inplace(ptr, old_size, size, align)
    }

    #[inline]
    pub unsafe fn deallocate(ptr: *mut u8, old_size: uint, align: uint) {
        rust_deallocate(ptr, old_size, align)
    }

    #[inline]
    pub fn usable_size(size: uint, align: uint) -> uint {
        unsafe { rust_usable_size(size, align) }
    }

    #[inline]
    pub fn stats_print() {
        unsafe { rust_stats_print() }
    }
}

pub fn oom() -> ! {
    // FIXME(#14674): This really needs to do something other than just abort
    //                here, but any printing done must be *guaranteed* to not
    //                allocate.
    unsafe { abort() }
}
