use kalloc::__rust_allocate as allocate;
use alloc::boxed::Box;
use core;
use core::slice;
use core::fmt;
use kbuf;
use collections;
use collections::vec::Vec;

use super::super::cpuio::IoPort;

use super::types;
use super::super::mem::*;
use super::super::sched;

const VRING_DESC_F_NEXT: u16 = 1; /* This marks a buffer as continuing via the next field. */
const VRING_DESC_F_WRITE: u16 = 2; /* This marks a buffer as write-only (otherwise read-only). */

#[repr(C, packed)]
struct BlockRequest {
  kind: u32,
  ioprio: u32,
  sector: u64,
}

// avail is: flags: u16, index u16, ring: [u16, length], used_event: u16
// used is: flags: u16, index u16, ring: [u64, length], avail_event: u16
fn vring_size(length: u16) -> usize {
  let descriptors = length as usize * core::mem::size_of::<types::Struct_vring_desc>();
  let avail = (length as usize + 3) * 2;
  let guest_write_side = descriptors + avail;

  let used = 3 * 2 + 8 * length as usize;
  let guest_read_side = used;

  align_up(guest_write_side, 0x1000) + guest_read_side
}

pub struct Virtqueue<'t> {
  length: u16,
  descriptors: &'t mut [Descriptor],
  avail: Avail<'t>,
  used: Used<'t>,

  _buffer: Box<[u8]>, // the actual memory for the references above
}

type Descriptor = types::Struct_vring_desc;

struct Avail<'t> {
  flags: &'t mut u16,
  idx: &'t mut u16,
  ring: &'t mut [u16],
}

struct Used<'t> {
  flags: &'t u16,
  idx: &'t u16,
  ring: &'t [u64],
}

impl<'t> Virtqueue<'t> {
  // queue_index is the index on the virtio device to initialize
  pub fn new(queue_index: u16, port: &IoPort) -> Self {
    // TODO: set queue_select

    // Determine how many descriptors the queue has, and allocate memory for the
    // descriptor table and the ring arrays.
    let length = port.read16(12);

    let memsize = vring_size(length);
    println!("Calculated vring size is {:x}", memsize);
    assert_eq!(0x1406, memsize); // sanity check

    // generate an aligned boxed slice for us to store the vring in
    let buf: Box<[u8]> = unsafe {
      let mem: *mut u8 = allocate(memsize, 0x1000);
      let vec = Vec::from_raw_parts(mem, memsize, memsize);
      vec.into_boxed_slice()
    };

    let desc: &mut [Descriptor];
    let avail;
    let used;

    {
      let (descbuf, after_desc) = buf.split_at(length as usize * core::mem::size_of::<types::Struct_vring_desc>());
      desc = unsafe {
        slice::from_raw_parts_mut(
          core::mem::transmute(descbuf.as_ptr()),
          length as usize)
      };

      let (availbuf, after_avail) = after_desc.split_at((length as usize + 3) * 2);
      avail = unsafe { Avail {
        flags: core::mem::transmute(availbuf.as_ptr()),
        idx: core::mem::transmute(availbuf.as_ptr().offset(2)),
        ring:
          slice::from_raw_parts_mut(
            core::mem::transmute(availbuf.as_ptr().offset(4)),
            length as usize)
      } };

      let guestlen = descbuf.len() + availbuf.len();
      let at = align_up(guestlen, 0x1000) - guestlen;

      let (blankspace, usedbuf) = after_avail.split_at(at);
      assert_eq!(usedbuf.len(), 3 * 2 + 8 * length as usize);

      used = unsafe { Used {
        flags: core::mem::transmute(usedbuf.as_ptr()),
        idx: core::mem::transmute(usedbuf.as_ptr().offset(2)),
        ring:
          slice::from_raw_parts(
            core::mem::transmute(usedbuf.as_ptr().offset(4)),
            length as usize)
      } };
    }

    println!("Descriptors at {:p}, avail at {:p}, used at {:p}", desc.as_ptr(), avail.flags, used.flags);

    // Now, tell the device where we placed the vring: take the kernel-space
    // address, get its physical address, turn it into a number, and shift right
    // by 12. It seems like this means that we "almost" support the 48-bit
    // effective address space on current x86_64 implementations.

    let physical_32 = physical_from_kernel(buf.as_ptr() as usize) as u32; // FIXME: not really a safe cast
    port.write32(8, physical_32 >> 12);

    Virtqueue {
      _buffer: buf,
      descriptors: desc,
      length: length,
      avail: avail,
      used: used,
    }
  }

  pub unsafe fn test(&mut self, port: &IoPort) -> bool {
    let hdrbuf = allocate(core::mem::size_of::<BlockRequest>(), 0x1);
    let databuf = allocate(512, 0x1);
    let donebuf = allocate(1, 0x1);

    // cor_printk("Telling virtio that target is at %p\n", (uint64_t)KTOP(payload));
    println!("target buffers: hdr@{:p}, data@{:p}, done@{:p}", hdrbuf, databuf, donebuf);

    // These entries describe a single logical buffer, composed of 3 separate physical buffers.
    // This separation is needed because a physical buffer can only be written to by one side.
    self.descriptors[0].addr = physical_from_kernel(hdrbuf as usize) as u64;
    self.descriptors[0].len = core::mem::size_of::<BlockRequest>() as u32;
    self.descriptors[0].flags = VRING_DESC_F_NEXT;
    self.descriptors[0].next = 1;

    self.descriptors[1].addr = physical_from_kernel(databuf as usize) as u64;
    self.descriptors[1].len = 512;
    self.descriptors[1].flags = VRING_DESC_F_NEXT | VRING_DESC_F_WRITE;
    self.descriptors[1].next = 2;

    self.descriptors[2].addr = physical_from_kernel(donebuf as usize) as u64;
    self.descriptors[2].len = 1;
    self.descriptors[2].flags = VRING_DESC_F_WRITE;

    // Okay, this was the slow setup part. Now we get to actually have fun using
    // these buffers. Firing off an actual I/O request involves these steps:
    // - Find a free header+payload+done buffer (in our case we only have one,
    //   so that's cool)
    // - Fill in the written-by-us part; in the block-device case, that means
    //   the request metadata header
    let hdr: &mut BlockRequest = core::mem::transmute(hdrbuf);
    hdr.kind = 0; // 0=read
    hdr.ioprio = 1; // prio
    hdr.sector = 0; // first sector of the disk

    *donebuf = 17; // debugging marker != 0, so that we can check if it worked

    // - Put the buffer into the virtqueue's "avail" array (the index-0 is actually
    //   `idx % qsz`, which wraps around after we've filled the avail array once,
    //   the value-0 is the index into the descriptor table above)
    self.avail.ring[0] = 0;

    // - Now, place a memory barrier so the above read is seen for sure
    core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);

    // - Now, tell the device which index into the array is the highest available one
    *self.avail.idx = 1;

    // // For reference, print out the current number of items available in the
    // // "processed" part of the ring; this should be 0, since nothing has been
    // // processed by the device yet.
    // cor_printk("before: %x\n", used->idx);
    println!("Number of readable queues before kick: {}", self.used.idx);
    if *self.used.idx != 0 {
      return false;
    }

    // Finally, we "kick" the device to tell it that it should look for
    // something to do. We could probably skip doing this and just wait for a
    // while; even after a kick, there's no guarantee that the request will have
    // been processed. The actual notification about "I did a thing, please go
    // check" will in practice be delivered back to us via an interrupt.

    // In reality, we'd now yield and wait until we receive virtio's interrupt that notifies
    // us of the completion of our request. We could also just spin for a bit.
    // Interestingly, this doesn't even seem to be required under QEMU/OS X.
    // Likely, the I/O write above directly triggers QEMU's virtio host driver
    // to execute the request. Obviously, this is completely undefined
    // behaviour we're relying on here, but let's just skip the wait while we
    // can.

    while *self.used.idx == 0 {
      port.write16(16, 0);
      sched::park_until_irq(0x2b);
    }

    // Now, magically, this index will have changed to "1" to indicate that
    // the device has processed our request buffer.

    println!("Number of readable queues after fake-wait: {}", self.used.idx);
    assert_eq!(1, *self.used.idx);

    println!("Virtio call completed, retval={}", *donebuf);
    if *donebuf != 0 { // retval of 0 indicates success
      return false;
    }

    // On success, the "payload" part of the buffer will contain the 512 read bytes.
    let data = slice::from_raw_parts_mut(databuf, 512);
    let needle = "DISK";
    let head = &data[0..20];

    let is_text = match core::str::from_utf8(head) {
      Ok(s) => {
        if &s.as_bytes()[0..needle.len()] == needle.as_bytes() {
          println!("text! disk contains: {}",s);
          true
        } else {
          println!("no text in disk header.");
          false
        }
      }
      Err(e) => {
        println!("expected disk header 'DISK', got invalid utf8: {:?}, {:?}", head, e);
        false
      }
    };

    let is_mbr = {
      data[510] == 0x55 && data[511] == 0xaa
    };

    is_text || is_mbr
  }
}
