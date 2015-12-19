use kalloc::__rust_allocate as allocate;
use alloc::boxed::Box;
use core;
use core::slice;
use core::fmt;
use kbuf;
use collections;
use collections::vec::Vec;
use core::borrow::BorrowMut;
use core::mem;
use super::super::mem::*;
use super::super::sched;

use super::BlockRequest;

const VRING_DESC_F_NEXT: u16 = 1; /* This marks a buffer as continuing via the next field. */
const VRING_DESC_F_WRITE: u16 = 2; /* This marks a buffer as write-only (otherwise read-only). */

// avail is: flags: u16, index u16, ring: [u16, length], used_event: u16
// used is: flags: u16, index u16, ring: [u64, length], avail_event: u16
fn size(length: u16) -> usize {
  let descriptors = length as usize * core::mem::size_of::<Descriptor>();
  let avail = (length as usize + 3) * 2;
  let guest_write_side = descriptors + avail;

  let used = 3 * 2 + 8 * length as usize;
  let guest_read_side = used;

  align_up(guest_write_side, 0x1000) + guest_read_side
}

type BufferToken = u16;

struct Descriptor {
  addr: u64,
  len: u32,
  flags: u16,
  next: BufferToken,
}

pub struct Vring {
  pub address: usize,
  length: usize,
  buf: Box<[u8]>,

  next_descriptor_i: u16,
  next_to_read: u16,

  // These are stored in `buf`.
  desc: *mut Descriptor,
  avail: Avail,
  used: Used,
}

impl fmt::Debug for Vring {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(vring)")
    }
}

struct Avail {
  flags: *mut u16,
  idx: *mut u16,
  ring: *mut u16,
}

struct Used {
  flags: *mut u16,
  idx: *mut u16,
  ring: *mut u64,
}

impl Vring {
  pub fn new(length: u16) -> Vring {
    let memsize = size(length);
    println!("Calculated vring size is {:x}", memsize);
    assert_eq!(0x1406, memsize); // sanity check

    // generate an aligned boxed slice for us to store the vring in
    let address : usize;
    let buf: Box<[u8]> = unsafe {
      let mem: *mut u8 = allocate(memsize, 0x1000);
      address = mem as usize;
      let vec = Vec::from_raw_parts(mem, memsize, memsize);
      vec.into_boxed_slice()
    };

    let desc: *mut Descriptor;
    let avail;
    let used;
    {
      let (descbuf, after_desc) = buf.split_at(length as usize * core::mem::size_of::<Descriptor>());
      desc = unsafe { core::mem::transmute(descbuf.as_ptr()) };

      let (availbuf, after_avail) = after_desc.split_at((length as usize + 3) * 2);
      avail = unsafe { Avail {
        flags: core::mem::transmute(availbuf.as_ptr()),
        idx: core::mem::transmute(availbuf.as_ptr().offset(2)),
        ring: core::mem::transmute(availbuf.as_ptr().offset(4)),
      } };

      let guestlen = descbuf.len() + availbuf.len();
      let at = align_up(guestlen, 0x1000) - guestlen;

      let (blankspace, usedbuf) = after_avail.split_at(at);
      assert_eq!(usedbuf.len(), 3 * 2 + 8 * length as usize);

      used = unsafe { Used {
        flags: core::mem::transmute(usedbuf.as_ptr()),
        idx: core::mem::transmute(usedbuf.as_ptr().offset(2)),
        ring: core::mem::transmute(usedbuf.as_ptr().offset(4)),
      } };
    }
    unsafe {
      *avail.idx = 0;
      *avail.flags = 0;
    }

    Vring { next_to_read: 0, next_descriptor_i: 0, length: length as usize, buf: buf, address: address, desc: desc, avail: avail, used: used }
  }

  pub fn enqueue_rww(&mut self, hdr: &[u8], data: &mut [u8], done: &mut [u8]) -> Result<BufferToken, ()> {
    let desc = unsafe { slice::from_raw_parts_mut(self.desc, self.length) };

    let i = self.next_descriptor_i as usize;
    // These entries describe a single logical buffer, composed of 3 separate physical buffers.
    // This separation is needed because a physical buffer can only be written to by one side.
    desc[i].addr = physical_from_kernel(hdr.as_ptr() as usize) as u64;
    desc[i].len = hdr.len() as u32;
    desc[i].flags = VRING_DESC_F_NEXT;
    desc[i].next = (i+1) as u16;

    desc[i+1].addr = physical_from_kernel(data.as_ptr() as usize) as u64;
    desc[i+1].len = data.len() as u32;
    desc[i+1].flags = VRING_DESC_F_NEXT | VRING_DESC_F_WRITE;
    desc[i+1].next = (i+2) as u16;

    desc[i+2].addr = physical_from_kernel(done.as_ptr() as usize) as u64;
    desc[i+2].len = done.len() as u32;
    desc[i+2].flags = VRING_DESC_F_WRITE;

    self.next_descriptor_i = self.next_descriptor_i + 3;

    // Put the buffer into the virtqueue's "avail" array (the index-0 is actually
    // `idx % qsz`, which wraps around after we've filled the avail array once,
    // the value-0 is the index into the descriptor table above)
    let availr = unsafe{ slice::from_raw_parts_mut(self.avail.ring, self.length) };
    unsafe { availr[*self.avail.idx as usize] = i as u16; }

    // Now, place a memory barrier so the above read is seen for sure
    core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);

    // Now, tell the device which index into the array is the highest available one
    unsafe { *self.avail.idx = *self.avail.idx + 1; }
    println!("Next buffer avail is: {}",unsafe{*self.avail.idx});

    Ok(0 as BufferToken)
  }

  // TODO: incorrect (index is not number of available bufs)
  pub fn take(&mut self) -> Option<BufferToken> {
    if unsafe { *self.used.idx as BufferToken } > self.next_to_read {
      println!("Taking buffer {} from used",self.next_to_read);
      self.next_to_read = self.next_to_read + 1;
      Some(0 as BufferToken)
    } else {
      None
    }
  }
}
