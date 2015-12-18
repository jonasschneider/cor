mod types;

use cpuio;

use kalloc::__rust_allocate as allocate;
use alloc::boxed::Box;
use core;
use core::slice;
use core::fmt;
use kbuf;
use collections;
use collections::vec::Vec;
use super::sched;


mod common_virtio {
  #[derive(Debug)]
  pub struct virtqueue<'t> {
    pub buf : ::kbuf::Buf<'t>
  }
}


#[derive(Debug)]
pub struct Device<'t> {
  io_base: cpuio::Port,
  q:  common_virtio::virtqueue<'t>
}

#[derive(Debug)]
pub enum Error {
  VirtioHandshakeFailure,
  NoDiskMarker,
}

const VIRTIO_STATUS_ACKNOWLEDGE: u8 = 1;
const VIRTIO_STATUS_DRIVER: u8 = 2;
const VIRTIO_STATUS_DRIVER_OK: u8 = 4;
const VIRTIO_STATUS_FAILED: u8 = 128;

const VRING_DESC_F_NEXT: u16 = 1; /* This marks a buffer as continuing via the next field. */
const VRING_DESC_F_WRITE: u16 = 2; /* This marks a buffer as write-only (otherwise read-only). */

fn physical_from_kernel(kernel: usize) -> usize {
  kernel & (0x0000008000000000-1)
}

#[repr(C, packed)]
struct BlockRequest {
  kind: u32,
  ioprio: u32,
  sector: u64,
}

pub unsafe fn init<'t>(io: u16) -> Result<Device<'t>, Error> {
  // We can now talk to the actual virtio device
  // via the CPU's I/O pins directly. A couple of helpful references:
  //
  // http://ozlabs.org/~rusty/virtio-spec/virtio-0.9.5.pdf
  //     This is the actual virtio spec.
  //
  // http://ozlabs.org/~rusty/virtio-spec/virtio-paper.pdf
  //     This is an academic paper describing the virtio design and architecture,
  //     and how a virtqueue works and is implemented.
  //
  // https://www.freebsd.org/cgi/man.cgi?query=virtio&sektion=4
  //     This is actually a FreeBSD manpage that gives a pretty good high-
  //     level overview of how the guest kernel usually interacts with the
  //     virtio interfaces and how it presents them to the guest OS's file
  //     system.

  let mut state = 0u8;
  println!("Initializing virtio block device starting at ioport {}..", io);
  cpuio::write8(io+18, state);

  state = state | VIRTIO_STATUS_ACKNOWLEDGE;
  cpuio::write8(io+18, state);

  state = state | VIRTIO_STATUS_DRIVER;
  cpuio::write8(io+18, state);

  // Feature negotiation
  let offered_featureflags = unsafe { cpuio::read16(io+0) };
  println!("The device offered us these feature bits: {:?}", offered_featureflags);
  // In theory, we'd do `negotiated = offered & supported`; we don't actually
  // support any flags, so we can just set 0.
  cpuio::write16(io+4, 0);

  // Now comes the block-device-specific setup.
  // (The configuration of a single virtqueue isn't device-specific though; it's the same
  // for i.e. the virtio network controller)

  // Discover virtqueues; the block devices only has one
  cpuio::write16(io+4, 0);
  if cpuio::read16(io+4) != 0 {
    return Err(Error::VirtioHandshakeFailure)
  }

  // Determine how many descriptors the queue has, and allocate memory for the
  // descriptor table and the ring arrays.
  let qsz = cpuio::read16(io+12) as usize;

  // size_t rsize = vring_size(qsz, 0x1000);
  // cor_printk("virtio's macros say that means a buffer size of %x\n", rsize);
  let rsize = 0x1406;

  // Align the start of the in-memory vring to a page boundary.
  // FIXME: we are leaking this
  // void *buf = tkalloc(rsize, "virtio vring", 0x1000); // lower align to page boundary
  let buf = allocate(rsize, 0x1000);

  // struct vring_desc *descriptors = (struct vring_desc*)buf;
  // struct vring_avail *avail = buf + qsz*sizeof(struct vring_desc);
  // struct vring_used *used = (struct vring_used*)ALIGN((uint64_t)avail+sizeof(struct vring_avail), 0x1000);

  // The first thing in the buffer are the descriptors.
  let descriptors : &mut [types::Struct_vring_desc] =
    slice::from_raw_parts_mut(core::mem::transmute(buf), qsz);

  // These address calculations are nontrivial because the vring is designed so that the
  // vring_avail and vring_used structs are on different pages.
  let after_descriptors = buf.offset((qsz * core::mem::size_of::<types::Struct_vring_desc>()) as isize);

  let avail: &mut types::Struct_vring_avail = core::mem::transmute(after_descriptors);

  let location_of_used = buf.offset(0x1000); // FIXME: this is correct, but ugly

  let used: &mut types::Struct_vring_used = core::mem::transmute(location_of_used);

  println!("R descriptors at {:p}", descriptors.as_ptr());
  println!("R avail       at {:p}", avail);
  println!("R used        at {:p}", used);

  // Now, tell the device where we placed the vring: take the kernel-space
  // address, get its physical address, turn it into a number, and shift right
  // by 12. It seems like this means that we "almost" support the 48-bit
  // effective address space on current x86_64 implementations.

  let physical_32 = physical_from_kernel(buf as usize) as u32; // FIXME: not really a safe cast
  cpuio::write32(io+8, physical_32 >> 12);

  // Tell the device we're done setting it up

  state = state | VIRTIO_STATUS_DRIVER_OK;
  cpuio::write8(io+18, state);

  println!("Device state is now: {}", state);

  // This completes the init sequence; we can know use the virtio device!

  // We control the virtual block device by sending pointers to  buffers to
  // the outside world, together with some metadata about e.g. the number of
  // the sector we want to read. The device then pops off these requests of
  // the virtqueue, and the read data magically appears in our buffer. (As I
  // understand it, pretty much like DMA.)
  //
  // The implementation of this concept isn't as simple as it could be, due to
  // performance reasons. It's actually a two-step process. First, we set up a
  // "descriptor table" which lists the buffers that we've allocated for using
  // with the virtio device, and whether this is a buffer that we write to or
  // one the hypervisor writes to (these are mutually exclusive.) This,
  // together with the buffer allocation itself, is the slow part; however, it
  // only has to be done very infrequently, i.e. when changing configurations.
  // In our trivial setup, we only need to do it once here.

  // struct virtio_blk_outhdr *hdr = (struct virtio_blk_outhdr *)tkalloc(sizeof(struct virtio_blk_outhdr), "virtio_blk request header", 0x10);
  // void *payload = tkalloc(512, "virtio_blk data buffer ", 0x10);
  // char *done = tkalloc(1, "virtio_blk status indicator ", 0x10);
  let hdrbuf = allocate(core::mem::size_of::<BlockRequest>(), 0x10);
  let databuf = allocate(512, 0x10);
  let donebuf = allocate(1, 0x10);

  // cor_printk("Telling virtio that target is at %p\n", (uint64_t)KTOP(payload));
  println!("target buffers: hdr@{:p}, data@{:p}, done@{:p}", hdrbuf, databuf, donebuf);

  // These entries describe a single logical buffer, composed of 3 separate physical buffers.
  // This separation is needed because a physical buffer can only be written to by one side.
  descriptors[0].addr = physical_from_kernel(hdrbuf as usize) as u64;
  descriptors[0].len = core::mem::size_of::<BlockRequest>() as u32;
  descriptors[0].flags = VRING_DESC_F_NEXT;
  descriptors[0].next = 1;

  descriptors[1].addr = physical_from_kernel(databuf as usize) as u64;
  descriptors[1].len = 512;
  descriptors[1].flags = VRING_DESC_F_NEXT | VRING_DESC_F_WRITE;
  descriptors[1].next = 2;

  descriptors[2].addr = physical_from_kernel(donebuf as usize) as u64;
  descriptors[2].len = 1;
  descriptors[2].flags = VRING_DESC_F_WRITE;


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

  *donebuf = 17; // debugging marker, so that we can check if it worked

  // - Put the buffer into the virtqueue's "avail" array (the index-0 is actually
  //   `idx % qsz`, which wraps around after we've filled the avail array once,
  //   the value-0 is the index into the descriptor table above)
  avail.ring = 0 as *mut u16; // FIXME: wait, what?


  // - Now, place a memory barrier so the above read is seen for sure
  core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);

  // - Now, tell the device which index into the array is the highest available one
  avail.idx = 1;

  // // For reference, print out the current number of items available in the
  // // "processed" part of the ring; this should be 0, since nothing has been
  // // processed by the device yet.
  // cor_printk("before: %x\n", used->idx);
  println!("Number of readable queues before kick: {}", used.idx);
  if used.idx != 0 {
    return Err(Error::VirtioHandshakeFailure);
  }

  // - Finally, "kick" the device to tell it that it should look for something
  //   to do. We could probably skip doing this and just wait for a while;
  //   even after a kick, there's no guarantee that the request will have been
  //   processed. The actual notification about "I did a thing, please go
  //   check" will in practice be delivered back to us via an interrupt.


  // Done! The request has been dispatched and the host informed that it has
  // some work queued.

  // In reality, we'd now yield and wait until we receive virtio's interrupt that notifies
  // us of the completion of our request. We could also just spin for a bit.
  // Interestingly, this doesn't even seem to be required under QEMU/OS X.
  // Likely, the I/O write above directly triggers QEMU's virtio host driver
  // to execute the request. Obviously, this is completely undefined
  // behaviour we're relying on here, but let's just skip the wait while we
  // can.

  // Now, magically, this index will change to "1" to indicate that
  // the device has processed our request buffer.

  while used.idx == 0 {
    cpuio::write16(io+16, 0);
    sched::park_until_irq(0x2b);
  }

  println!("Number of readable queues after fake-wait: {}", used.idx);

  println!("Virtio call completed, retval={}", *donebuf);
  if *donebuf != 0 { // retval of 0 indicates success
    return Err(Error::VirtioHandshakeFailure);
  }

  // On success, the "payload" part of the buffer will contain the 512 read bytes.
  let data = slice::from_raw_parts_mut(databuf, 512);
  let needle = "DISK";
  let head = &data[0..20];

  match core::str::from_utf8(head) {
    Ok(s) => {
      if &s.as_bytes()[0..needle.len()] != needle.as_bytes() {
        println!("invalid disk header. expected {:?} ('DISK'), got {:?}", needle.as_bytes(), s.as_bytes());
        return Err(Error::NoDiskMarker);
      }
      println!("disk contains: {}",s);
    }
    Err(e) => {
      println!("expected disk header 'DISK', got invalid utf8: {:?}, {:?}", head, e);
      return Err(Error::NoDiskMarker);
    }
  }

  // And this, dear reader, is how (surprisingly) easy it is to talk to a
  // virtio block device! Of course, this is just a spike implementation,
  // there could be buffer management, request
  // multiplexing/reordering/scheduling going on.

  println!("Virtio-blk device successfully initialized and tested!");

  let mybuf = kbuf::new("a buffer");
  let theq = common_virtio::virtqueue{buf: mybuf};

  Ok(Device { q: theq, io_base: io, })
}
