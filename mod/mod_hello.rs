#![crate_type="staticlib"]
#![no_std]
#![feature(globs,lang_items)]

extern crate core;
extern crate libc;

extern {
  fn abort() -> !;
}

#[lang = "owned_box"]
pub struct Box<T>(*mut T);

#[lang="exchange_malloc"]
unsafe fn allocate(size: uint, _align: uint) -> *mut u8 {
  let p = libc::malloc(size as libc::size_t) as *mut u8;

  // malloc failed
  if p as uint == 0 {
      abort();
  }

  p
}
#[lang="exchange_free"]
unsafe fn deallocate(ptr: *mut u8, _size: uint, _align: uint) {
  libc::free(ptr as *mut libc::c_void)
}

#[start]
#[no_mangle]
pub unsafe fn hello_main() {
  let x = box 1i;

  let v = *x;
  if v == 1i {
    abort();
  }
}

#[lang = "stack_exhausted"] extern fn stack_exhausted() {}
#[lang = "eh_personality"] extern fn eh_personality() {}
#[lang = "panic_fmt"] fn panic_fmt() -> ! { loop {} }
