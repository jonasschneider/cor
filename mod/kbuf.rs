use core::mem;
use kalloc::__rust_allocate as allocate;
use kalloc::__rust_deallocate as deallocate;
use core::slice;
use core;

#[derive(Debug)]
pub struct Buf<'buflife> {
  pub s : &'buflife mut[u8],
  pub original_mem : *mut u8, // FIXME this should not be public, I guess
  original_size: usize,
  original_align: usize,
}

pub fn new<'buflife>(name : &'buflife str) -> Buf<'buflife> {
  let size = 0x4000;
  let align = 0x100;
  let mem = unsafe { allocate(size, align) };
  let memptr : *mut u8 = unsafe { mem::transmute(mem) };
  let slice : &mut[u8] = unsafe { slice::from_raw_parts_mut(memptr, 0x4000) };
  Buf{s: slice, original_mem: mem, original_size: size, original_align: align}
}

impl<'buflife> core::ops::Drop for Buf<'buflife> {
    fn drop(&mut self) {
      println!("freeing a kbuf");
      unsafe { deallocate(self.original_mem, self.original_size, self.original_align) }
    }
}
