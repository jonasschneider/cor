use kalloc::__rust_allocate as allocate;
use alloc::boxed::Box;
use core;
use core::slice;
use core::fmt;
use kbuf;
use collections;
use collections::vec::Vec;
use core::borrow::{BorrowMut,Borrow};

use cpuio::IoPort;

use super::vring;
use super::vring::Descriptor;
use mem::*;
use sched;

const VRING_DESC_F_NEXT: u16 = 1; /* This marks a buffer as continuing via the next field. */
const VRING_DESC_F_WRITE: u16 = 2; /* This marks a buffer as write-only (otherwise read-only). */

// queue_index is the index on the virtio device to initialize
pub fn setup(queue_index: u16, port: &mut IoPort) -> (BufferPool, vring::Used) {
  // Set queue_select
  port.write16(14, queue_index);

  // Determine how many descriptors the queue has, and allocate memory for the
  // descriptor table and the ring arrays.
  let length = port.read16(12);

  let (address, mut avail, mut used) = vring::setup(length);

  // Now, tell the device where we placed the vring: take the kernel-space
  // address, get its physical address, turn it into a number, and shift right
  // by 12. It seems like this means that we "almost" support the 48-bit
  // effective address space on current x86_64 implementations.

  let physical_32 = physical_from_kernel(address as usize) as u32; // FIXME: not really a safe cast
  port.write32(8, physical_32 >> 12);

  (BufferPool{avail: avail, next_descriptor_i: 0}, used)
}

#[derive(Debug)]
pub struct BufferPool {
  pub avail: vring::Avail,
  next_descriptor_i: u16,
}

impl BufferPool {
  pub fn enqueue_rww(&mut self, hdr: &[u8], data: &mut [u8], done: &mut [u8]) -> Result<u16, ()> {
    let i_ = self.next_descriptor_i;
    let i = i_ as usize;

    // These entries describe a single logical buffer, composed of 3 separate physical buffers.
    // This separation is needed because a physical buffer can only be written to by one side.
    self.avail.write_descriptor_at(i, Descriptor {
      addr: physical_from_kernel(hdr.as_ptr() as usize) as u64,
      len: hdr.len() as u32,
      flags: VRING_DESC_F_NEXT,
      next: (i+1) as u16,
    });

    self.avail.write_descriptor_at(i+1, Descriptor {
      addr: physical_from_kernel(data.as_ptr() as usize) as u64,
      len: data.len() as u32,
      flags: VRING_DESC_F_NEXT | VRING_DESC_F_WRITE,
      next: (i+2) as u16,
    });

    self.avail.write_descriptor_at(i+2, Descriptor {
      addr: physical_from_kernel(done.as_ptr() as usize) as u64,
      len: done.len() as u32,
      flags: VRING_DESC_F_WRITE,
      next: 0,
    });

    self.next_descriptor_i = self.next_descriptor_i + 3;
    self.avail.add_to_ring(i_);
    Ok(i_)
  }
}
