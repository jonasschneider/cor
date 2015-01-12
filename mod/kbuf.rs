
use core::mem;
use myheap::allocate;
use myheap::deallocate;
use core::slice;
use core;

pub struct Buf<'buflife> {
  pub s : &'buflife mut[u8],
  pub original_mem : *mut u8, // FIXME this should not be public, I guess
  original_size: uint,
  original_align: uint,
}

pub fn new<'buflife>(name : &'buflife str) -> Buf<'buflife> {
  let size = 0x1000;
  let align = 0x10;
  let mem = unsafe { allocate(size, align) };
  let memptr : &*mut u8 = unsafe { mem::transmute(&mem) };
  let slice : &mut[u8] = unsafe { slice::from_raw_mut_buf(memptr, 512) };
  Buf{s: slice, original_mem: mem, original_size: size, original_align: align}
}

#[unsafe_destructor]
impl<'buflife> core::ops::Drop for Buf<'buflife> {
    fn drop(&mut self) {
      println!("freeing a kbuf");
      unsafe { deallocate(self.original_mem, self.original_size, self.original_align) }
    }
}
