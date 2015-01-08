use cpuio;

use boxed::Box;
use core::slice;
use core::fmt;

mod bytebuf {
  use core::mem;
  use myheap::allocate;
  use myheap::deallocate;
  use core::slice;
  use core;

  pub struct Buf<'buflife> {
    pub s : &'buflife mut[u8],
    original_mem : *mut u8,
    original_size: uint,
    original_align: uint,
  }

  pub fn new<'buflife>(name : &'buflife str) -> Buf<'buflife> {
    let size = 512;
    let align = 1;
    let mem = unsafe { allocate(size, align) };
    let memptr : &*mut u8 = unsafe { mem::transmute(&mem) };
    let slice : &mut[u8] = unsafe { slice::from_raw_mut_buf(memptr, 512) };
    Buf{s: slice, original_mem: mem, original_size: size, original_align: align}
  }

  #[unsafe_destructor]
  impl<'buflife> core::ops::Drop for Buf<'buflife> {
      fn drop(&mut self) {
        unsafe { deallocate(self.original_mem, self.original_size, self.original_align) }
      }
  }
}

mod common_virtio {
  pub struct virtqueue<'dev_life> {
    pub buf : ::block::virtio::bytebuf::Buf<'dev_life>
  }
}

struct virtio_blkdev<'dev_life> {
  io_base: cpuio::Port,
  q:  common_virtio::virtqueue<'dev_life>
}


pub fn init<'devlife>(io_base : &'devlife u16) -> Box<virtio_blkdev> {
  let io = *io_base;

  let mybuf = bytebuf::new("a buffer");
  let theq = common_virtio::virtqueue{buf: mybuf};
  let dev = box virtio_blkdev{
    q: theq,
    io_base: io
  };


  let mut state = 0;
  println!("Initializing virtio block device...");
  unsafe { cpuio::write8(io+18, state); }

  // ack
  state = state | 1;
  unsafe { cpuio::write8(io+18, state); }

  // drive
  state = state | 2;
  unsafe { cpuio::write8(io+18, state); }

  // Feature negotiation
  let offered_featureflags = unsafe { cpuio::read16(io+0) };
  println!("The device offered us these feature bits: {}", offered_featureflags);
  // In theory, we'd do `negotiated = offered & supported`; we don't actually
  // support any flags, so we can just set 0.
  unsafe {cpuio::write16(io+4, 0); }

//   // Discover virtqueues; the block devices only has one
//   iowrite16(io_base+14, 0); // select first queue
//   if(ioread16(io_base+14) != 0) {
//     cor_panic("Failed to select queue0");
//   }

//   // Determine how many descriptors the queue has, and allocate memory for the
//   // descriptor table and the ring arrays.
//   uint16_t qsz = ioread16(io_base+12);
//   cor_printk("virtqueue has sizenum=%x\n", qsz);
//   size_t rsize = vring_size(qsz, 0x1000);
//   cor_printk("virtio's macros say that means a buffer size of %x\n", rsize);

//   void *buf = tkalloc(rsize, "virtio vring", 0x1000); // lower align to page boundary

//   for(unsigned int i = 0; i < rsize; i++) { // TODO: memzero
//     *((char*)(buf+i)) = 0;
//   }

//   // The address calculation is nontrivial because the vring is designed so that the
//   // vring_avail and vring_used structs are on different pages.
//   struct vring_desc *descriptors = (struct vring_desc*)buf;
//   struct vring_avail *avail = buf + qsz*sizeof(struct vring_desc);
//   struct vring_used *used = (struct vring_used*)ALIGN((uint64_t)avail+sizeof(struct vring_avail), 0x1000);

//   cor_printk("descriptors at %p\n", descriptors);
//   cor_printk("avail       at %p\n", avail);
//   cor_printk("used        at %p\n", used);

//   // Now, tell the device where we placed the vring: take the kernel-space
//   // address, get its physical address, turn it into a number, and shift right
//   // by 12. It seems like this means that we "almost" support the 48-bit
//   // effective address space on current x86_64 implementations.
//   sysOutLong (io_base+8, (uint32_t)(((ptr_t)KTOP(buf)) /4096));

//   // TODO: The spec says that we can do something with MSI-X here, whatever

//   // Tell the device we're done setting it up
//   state |= VIRTIO_STATUS_DRIVER_OK;
//   cor_outb(state, io_base+18);
//   cor_printk("Device state set to: %x\n", state); // this should be 7 now

//   // This completes the init sequence; we can know use the virtio device!

//   // We control the virtual block device by sending pointers to  buffers to
//   // the outside world, together with some metadata about e.g. the number of
//   // the sector we want to read. The device then pops off these requests of
//   // the virtqueue, and the read data magically appears in our buffer. (As I
//   // understand it, pretty much like DMA.)
//   //
//   // The implementation of this concept isn't as simple as it could be, due to
//   // performance reasons. It's actually a two-step process. First, we set up a
//   // "descriptor table" which lists the buffers that we've allocated for using
//   // with the virtio device, and whether this is a buffer that we write to or
//   // one the hypervisor writes to (these are mutually exclusive.) This,
//   // together with the buffer allocation itself, is the slow part; however, it
//   // only has to be done very infrequently, i.e. when changing configurations.
//   // In our trivial setup, we only need to do it once here.
//   struct virtio_blk_outhdr *hdr = (struct virtio_blk_outhdr *)tkalloc(sizeof(struct virtio_blk_outhdr), "virtio_blk request header", 0x10);
//   void *payload = tkalloc(512, "virtio_blk data buffer ", 0x10);
//   char *done = tkalloc(1, "virtio_blk status indicator ", 0x10);

//   cor_printk("Telling virtio that target is at %p\n", (uint64_t)KTOP(payload));

//   // These entries actually describe only a single logical buffer, however,
//   // that buffer is composed of 3 separate buffers. (This separation is required
//   // because a physical buffer can only be written to by one side.)
//   descriptors[0].addr = (uint64_t)KTOP(hdr);
//   descriptors[0].len = sizeof(struct virtio_blk_outhdr);
//   descriptors[0].flags = VRING_DESC_F_NEXT;
//   descriptors[0].next = 1;

//   descriptors[1].addr = (uint64_t)KTOP(payload);
//   descriptors[1].len = 512;
//   descriptors[1].flags = VRING_DESC_F_NEXT | VRING_DESC_F_WRITE;
//   descriptors[1].next = 2;

//   descriptors[2].addr = (uint64_t)KTOP(done);
//   descriptors[2].len = 1;
//   descriptors[2].flags = VRING_DESC_F_WRITE;


//   // Okay, this was the slow setup part. Now we get to actually have fun using
//   // these buffers. Firing off an actual I/O request involves these steps:
//   // - Find a free header+payload+done buffer (in our case we only have one,
//   //   so that's cool)
//   // - Fill in the written-by-us part; in the block-device case, that means
//   //   the request metadata header
//   hdr->type = 0; // 0=read
//   hdr->ioprio = 1; // prio
//   hdr->sector = 0; // should be the MBR

//   // - TODO: Maybe somehow "reset" the part that's not written by us? Not sure
//   //   if we need, though
//   *done = 17; // debugging marker, so that we can check if it worked

//   // - Put the buffer into the virtqueue's "avail" array (the index-0 is actually
//   //   `idx % qsz`, which wraps around after we've filled the avail array once,
//   //   the value-0 is the index into the descriptor table above)
//   avail->ring[0] = 0;

//   // - Now, place a memory barrier so the above read is seen for sure
//   __asm__ volatile ( "" : : : "memory");

//   // - Now, tell the device which index into the array is the highest available one
//   avail->idx = 1;

//   // For reference, print out the current number of items available in the
//   // "processed" part of the ring; this should be 0, since nothing has been
//   // processed by the device yet.
//   cor_printk("before: %x\n", used->idx);

//   // - Finally, "kick" the device to tell it that it should look for something
//   //   to do. We could probably skip doing this and just wait for a while;
//   //   even after a kick, there's no guarantee that the request will have been
//   //   processed. The actual notification about "I did a thing, please go
//   //   check" will in practice be delivered to us via an interrupt. (I think)
//   iowrite16(io_base+16, 0);

//   __asm__ ("hlt");


//   // Now, in reality, we'd wait until we receive the aforementioned
//   // interrupt. However, I haven't set up anything using interrupts yet. A
//   // cringeworthy "alternative" is just to busy loop for a while.

//   // Interestingly, this doesn't even seem to be required under QEMU/OS X.
//   // Likely, the I/O write above directly triggers QEMU's virtio host driver
//   // to execute the request. Obviously, this is completely unspecified
//   // behvaiour we're relying on here, but let's just skip the wait while we
//   // can.
//   //for(int i = 0; i < 100000000; i++);

//   // Now, magically, this index should have changed to "1" to indicate that
//   // the device has processed our request buffer.
//   cor_printk("after: %x\n", used->idx);
//   if(used->idx != 0) {
//     cor_printk("virtio call completed, ret=%u\n", *done);
//     if(*done == 0) { // 0 indicate success
//       // On success, the "payload" part of the buffer will contain the 512 read bytes.
//       char tbuf[21];
//       for(int i = 0; i < 20; i++) {
//         tbuf[i] = *(char*)(payload+i);
//       }
//       tbuf[20] = '\0';
//       cor_printk("ascii=%s\n", tbuf);
//     }
//   } else {
//     // this could still just be a race condition
//     cor_panic("virtio call didn't complete");
//   }

//   // And this, dear reader, is how (surprisingly) easy it is to talk to a
//   // virtio block device! Of course, this is just a spike implementation,
//   // there could be buffer management, request
//   // multiplexing/reordering/scheduling going on.

//   cor_printk("Done initializing the virtio block device\n");
// }

  return dev
}


impl<'a> fmt::Show for virtio_blkdev<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // The `f` value implements the `Writer` trait, which is what the
        // write! macro is expecting. Note that this formatting ignores the
        // various flags provided to format strings.
        write!(f, "some dev")
    }
}
