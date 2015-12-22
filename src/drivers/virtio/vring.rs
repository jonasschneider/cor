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
use mem::*;
use sched;

use byteorder::{ByteOrder,NativeEndian};

const VRING_DESC_F_NEXT: u16 = 1; /* This marks a buffer as continuing via the next field. */
const VRING_DESC_F_WRITE: u16 = 2; /* This marks a buffer as write-only (otherwise read-only). */

pub fn setup(length: u16) -> (*const u8, Avail, Used) {
  let (writesize, readsize) = size(length);
  let (writebuf, readbuf) = alloc_pagealigned(writesize, readsize);
  let address = writebuf.as_ptr();

  let avail = Avail{ mem: writebuf, qsz: length as usize };
  let used = Used{ mem: readbuf, qsz: length as usize, last_taken_index: None };

  (address, avail, used)
}

// avail is: flags: u16, index u16, ring: [u16, length], used_event: u16
// used is: flags: u16, index u16, ring: [u64, length], avail_event: u16
fn size(length: u16) -> (usize, usize) {
  let descriptors = length as usize * core::mem::size_of::<Descriptor>();
  let avail = (length as usize + 3) * 2;
  let guest_write_side = descriptors + avail;

  let used = 3 * 2 + 8 * length as usize;
  let guest_read_side = used;

  (guest_write_side, guest_read_side)
}

// Allocate two memory blocks, page-aligned, next to each other.
// The padding between the end of the first slice and the start of the second block
// is *not* allocated and has undefined contents.
// TODO: Zero this out!
fn alloc_pagealigned(size1: usize, size2: usize) -> (Box<[u8]>, Box<[u8]>) {
  unsafe {
    let mem1: *mut u8 = allocate(size1, 0x1000);
    let mem2: *mut u8 = allocate(size2, 0x1000);

    // make sure the allocator did what we expect
    assert_eq!(mem2 as usize, align_up(mem1 as usize + size1, 0x1000));

    (Vec::from_raw_parts(mem1, size1, size1).into_boxed_slice(),
     Vec::from_raw_parts(mem2, size2, size2).into_boxed_slice())
  }
}

pub struct Descriptor {
  pub addr: u64,
  pub len: u32,
  pub flags: u16,
  pub next: u16,
}

#[derive(Debug)]
pub struct Avail {
  pub mem: Box<[u8]>,
  qsz: usize,
}

// Optimizations could probably still break this. :(
// TODO make sure that the wrapping is not modulo u16, but modulo qsz
impl Avail {
  pub fn write_descriptor_at(&mut self, pos: usize, d: Descriptor) {
    NativeEndian::write_u64(&mut self.mem[pos*16..], d.addr);
    NativeEndian::write_u32(&mut self.mem[pos*16+8..], d.len);
    NativeEndian::write_u16(&mut self.mem[pos*16+12..], d.flags);
    NativeEndian::write_u16(&mut self.mem[pos*16+14..], d.next);
  }

  pub fn add_to_ring(&mut self, idx: u16) {
    let mut current_head = NativeEndian::read_u16(&self.mem[self.qsz*16+2..]);

    // Put the buffer into the virtqueue's "avail" array (the index-0 is actually
    // `idx % qsz`, which wraps around after we've filled the avail array once,
    // the value-0 is the index into the descriptor table above)
    NativeEndian::write_u16(&mut self.mem[self.qsz*16+4+(current_head as usize)*2..], idx);
    current_head = current_head.wrapping_add(1);

    // Now, place a memory barrier so the above write is seen for sure.. is that enough?
    core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);

    println!("Next buffer avail is: {}",current_head);

    // "Publish" the new buffer head position
    NativeEndian::write_u16(&mut self.mem[self.qsz*16+2..], current_head);
  }
}

#[derive(Debug)]
pub struct Used {
  mem: Box<[u8]>,
  qsz: usize,
  last_taken_index: Option<u16>,
}

impl Used {
  // TODO: not sure if this is correct
  // Return the descriptor index of the taken buffer, if any.
  pub fn take_from_ring(&mut self) -> Option<u16> {
    let current_head = NativeEndian::read_u16(&self.mem[2..]);

    let ring_index_to_take = match self.last_taken_index {
      None => if current_head != 0 { Some(0) } else { None },
      Some(last) => {
        if current_head != last.wrapping_add(1) { // There is *something* between the last thing we took and the head
          Some(last.wrapping_add(1)) // allow wraparound
        } else {
          None
        }
      }
    };

    match ring_index_to_take {
      Some(i) => {
        let descid = NativeEndian::read_u16(&self.mem[4+2*(i as usize)..]);
        self.last_taken_index = Some(i);
        println!("Taking buffer {} from index {}", descid, i);
        Some(i)
      },
      None => {
        println!("Nothing to take: head {}, last {:?}", current_head, self.last_taken_index);
        None
      }
    }
  }
}
