use kalloc::__rust_allocate as allocate;
use alloc::boxed::Box;
use core;
use core::slice;
use core::fmt;
use kbuf;
use collections;
use collections::vec::Vec;
use core::borrow::{BorrowMut,Borrow};

use super::super::cpuio::IoPort;

use super::vring;
use super::super::mem::*;
use super::super::sched;

const VRING_DESC_F_NEXT: u16 = 1; /* This marks a buffer as continuing via the next field. */
const VRING_DESC_F_WRITE: u16 = 2; /* This marks a buffer as write-only (otherwise read-only). */

#[repr(C, packed)]
pub struct BlockRequest {
  kind: u32,
  ioprio: u32,
  sector: u64,
}

pub struct Virtqueue {
  ring: vring::Vring,
}

impl Virtqueue {
  // queue_index is the index on the virtio device to initialize
  pub fn new(queue_index: u16, port: &IoPort) -> Self {
    // TODO: set queue_select

    // Determine how many descriptors the queue has, and allocate memory for the
    // descriptor table and the ring arrays.
    let length = port.read16(12);

    let ring = vring::Vring::new(length);

    // Now, tell the device where we placed the vring: take the kernel-space
    // address, get its physical address, turn it into a number, and shift right
    // by 12. It seems like this means that we "almost" support the 48-bit
    // effective address space on current x86_64 implementations.

    let physical_32 = physical_from_kernel(ring.address) as u32; // FIXME: not really a safe cast
    port.write32(8, physical_32 >> 12);

    Virtqueue {
      ring: ring
    }
  }

  pub fn test(&mut self, port: &IoPort) -> bool {
    let mut hdr = BlockRequest {
      kind: 0, // 0=read
      ioprio: 1, // prio
      sector: 0
    };
    let mut data = [0u8; 512];
    let mut done = [17u8; 1]; // != 0 for checking that it was set by the host

    {
      let hdrbuf: &[u8] = unsafe{ slice::from_raw_parts(core::mem::transmute(&hdr), core::mem::size_of::<BlockRequest>()) };
      self.ring.enqueue_rww(&hdrbuf, &mut data[..], &mut done[..]);
    }

    // Finally, we "kick" the device to tell it that it should look for
    // something to do. We could probably skip doing this and just wait for a
    // while; even after a kick, there's no guarantee that the request will have
    // been processed. The actual notification about "I did a thing, please go
    // check" will be delivered back to us via an interrupt.
    // Now we park ourselves until things change.

    while let None = self.ring.take() {
      port.write16(16, 0);
      sched::park_until_irq(0x2b);
    }

    // Now, magically, this index will have changed to "1" to indicate that
    // the device has processed our request buffer.

    println!("Virtio call completed, retval={}", done[0]);

    if done[0] != 0 { // retval of 0 indicates success
      return false;
    }

    // On success, the "payload" part of the buffer will contain the 512 read bytes.
    let needle = "DISK";
    let head = &data[0..20];

    let is_text = match core::str::from_utf8(head) {
      Ok(s) => {
        if &s.as_bytes()[0..needle.len()] == needle.as_bytes() {
          println!("Disk text: {}",s);
          true
        } else {
          false
        }
      }
      Err(e) => {
        false
      }
    };

    let is_mbr = {
      data[510] == 0x55 && data[511] == 0xaa
    };

    println!("Text:{} Mbr:{}", is_text, is_mbr);

    is_text || is_mbr
  }
}
