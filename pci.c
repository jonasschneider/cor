#include "common.h"
#include "io.h"
#include "sched.h"
#include "vendor/virtio.h"

// ref: http://wiki.osdev.org/PCI

void virtio_init(unsigned int ioport);

uint32_t sysInLong(uint16_t port) {
  uint32_t res;
  __asm__ (
    "in %%dx, %%eax\n"
    : "=a" (res)
    : "d" (port)
  );
  return res;
}

void sysOutLong(uint16_t port, uint32_t value) {
  __asm__ (
    "out %%eax, %%dx\n"
    :
    : "d" (port), "a" (value)
  );
}

uint16_t ioread16(uint16_t port) {
  uint16_t res;
  __asm__ (
    "in %%dx, %%ax\n"
    : "=a" (res)
    : "d" (port)
  );
  return res;
}

void iowrite16(uint16_t port, uint16_t value) {
  __asm__ (
    "out %%ax, %%dx\n"
    :
    : "d" (port), "a" (value)
  );
}


uint16_t pciConfigReadWord (uint8_t bus, uint8_t slot,
                           uint8_t func, uint8_t offset)
{
  uint32_t address;
  uint32_t lbus  = (uint32_t)bus;
  uint32_t lslot = (uint32_t)slot;
  uint32_t lfunc = (uint32_t)func;
  uint16_t tmp = 0;

  /* create configuration address as per Figure 1 */
  address = (uint32_t)((lbus << 16) | (lslot << 11) |
            (lfunc << 8) | (offset & 0xfc) | ((uint32_t)0x80000000));

  /* write out the address */
  sysOutLong (0xCF8, address);
  /* read in the data */
  /* (offset & 2) * 8) = 0 will choose the first word of the 32 bits register */
  tmp = (uint16_t)((sysInLong (0xCFC) >> ((offset & 2) * 8)) & 0xffff);
  return (tmp);
}

uint32_t pciConfigReadLong (uint8_t bus, uint8_t slot,
                           uint8_t func, uint8_t offset)
{
  uint32_t address;
  uint32_t lbus  = (uint32_t)bus;
  uint32_t lslot = (uint32_t)slot;
  uint32_t lfunc = (uint32_t)func;

  /* create configuration address as per Figure 1 */
  address = (uint32_t)((lbus << 16) | (lslot << 11) |
            (lfunc << 8) | (offset & 0xfc) | ((uint32_t)0x80000000));

  sysOutLong (0xCF8, address);

  return sysInLong(0xCFC);
}

void pciConfigWriteLong (uint8_t bus, uint8_t slot,
                           uint8_t func, uint8_t offset, uint32_t val)
{
  uint32_t address;
  uint32_t lbus  = (uint32_t)bus;
  uint32_t lslot = (uint32_t)slot;
  uint32_t lfunc = (uint32_t)func;

  /* create configuration address as per Figure 1 */
  address = (uint32_t)((lbus << 16) | (lslot << 11) |
            (lfunc << 8) | (offset & 0xfc) | ((uint32_t)0x80000000));

  // first set the value, because addres shas the enable/trigger bit
  sysOutLong (0xCFC, val);
  sysOutLong (0xCF8, address);
}

#define VIRTIO_STATUS_ACKNOWLEDGE 1
#define VIRTIO_STATUS_DRIVER 2
#define VIRTIO_STATUS_DRIVER_OK 4
#define VIRTIO_STATUS_FAILED 128

#pragma pack(push, 1)
struct virtio_blk_outhdr {
  uint32_t type;
  uint32_t ioprio;
  uint64_t sector;
};
#pragma pack(pop)

void virtio_init();

void setup_virtio(uint8_t bus, uint8_t slot, uint8_t function) {
  cor_printk("Found a virtio block device!\nThis is its configuration space:\n");
  for(int i = 0; i < 0x3c; i+=4) {
    cor_printk("%x = %x\n", i, pciConfigReadLong(bus, slot, function, i));
  }

  // Read out the I/O port location where we can talk to the device.
  // c.f. http://wiki.osdev.org/PCI#Base_Address_Registers.
  // For virtio devices, BAR0 is always an I/O register.
  // virtio also has a more modern memory-mapped configuration system,
  // but we won't use it here.
  uint32_t bar0 = pciConfigReadLong(bus, slot, function, 0x10);
  uint16_t io_base = bar0 & 0xFFFFFFFC;

  uint8_t irq = (uint8_t)(pciConfigReadLong(bus, slot, function, 0x3c) & 0xff);
  cor_printk("Virtio IRQ is %x\n", irq);

  // This is pretty much all we actually interface with PCI; once we have the
  // I/O base port, we're golden.

  cor_printk("Doing rust call thingie\n");
  //virtio_init(io_base);

  // Now comes the block-device-specific setup.
  // (The configuration of a single virtqueue isn't device-specific though; it's the same
  // for i.e. the virtio network controller)

  // Feature negotiation
  uint32_t offered_featureflags = sysInLong(io_base);
  cor_printk("The device offers these feature bits: %x\n", offered_featureflags);
  // In theory, we'd do `negotiated = offered & supported`; we don't actually
  // support any flags, so we can just set 0.
  sysOutLong(io_base+4, 0);

  // Discover virtqueues; the block devices only has one
  iowrite16(io_base+14, 0); // select first queue
  if(ioread16(io_base+14) != 0) {
    cor_panic("Failed to select queue0");
  }

  // Determine how many descriptors the queue has, and allocate memory for the
  // descriptor table and the ring arrays.
  uint16_t qsz = ioread16(io_base+12);
  cor_printk("virtqueue has sizenum=%x\n", qsz);
  size_t rsize = vring_size(qsz, 0x1000);
  cor_printk("virtio's macros say that means a buffer size of %x\n", rsize);

  void *buf = tkalloc(rsize, "virtio vring", 0x1000); // lower align to page boundary

  for(unsigned int i = 0; i < rsize; i++) { // TODO: memzero
    *((char*)(buf+i)) = 0;
  }

  // The address calculation is nontrivial because the vring is designed so that the
  // vring_avail and vring_used structs are on different pages.
  struct vring_desc *descriptors = (struct vring_desc*)buf;
  struct vring_avail *avail = buf + qsz*sizeof(struct vring_desc);
  struct vring_used *used = (struct vring_used*)ALIGN((uint64_t)avail+sizeof(struct vring_avail), 0x1000);

  cor_printk("descriptors at %p\n", descriptors);
  cor_printk("avail       at %p\n", avail);
  cor_printk("used        at %p\n", used);

  // Now, tell the device where we placed the vring: take the kernel-space
  // address, get its physical address, turn it into a number, and shift right
  // by 12. It seems like this means that we "almost" support the 48-bit
  // effective address space on current x86_64 implementations.
  sysOutLong (io_base+8, (uint32_t)(((ptr_t)KTOP(buf)) /4096));

  // TODO: The spec says that we can do something with MSI-X here, whatever

  // Tell the device we're done setting it up
  // state |= VIRTIO_STATUS_DRIVER_OK;
  // cor_outb(state, io_base+18);
  // cor_printk("Device state set to: %x\n", state); // this should be 7 now

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
  struct virtio_blk_outhdr *hdr = (struct virtio_blk_outhdr *)tkalloc(sizeof(struct virtio_blk_outhdr), "virtio_blk request header", 0x10);
  void *payload = tkalloc(512, "virtio_blk data buffer ", 0x10);
  char *done = tkalloc(1, "virtio_blk status indicator ", 0x10);

  cor_printk("Telling virtio that target is at %p\n", (uint64_t)KTOP(payload));

  // These entries actually describe only a single logical buffer, however,
  // that buffer is composed of 3 separate buffers. (This separation is required
  // because a physical buffer can only be written to by one side.)
  descriptors[0].addr = (uint64_t)KTOP(hdr);
  descriptors[0].len = sizeof(struct virtio_blk_outhdr);
  descriptors[0].flags = VRING_DESC_F_NEXT;
  descriptors[0].next = 1;

  descriptors[1].addr = (uint64_t)KTOP(payload);
  descriptors[1].len = 512;
  descriptors[1].flags = VRING_DESC_F_NEXT | VRING_DESC_F_WRITE;
  descriptors[1].next = 2;

  descriptors[2].addr = (uint64_t)KTOP(done);
  descriptors[2].len = 1;
  descriptors[2].flags = VRING_DESC_F_WRITE;


  // Okay, this was the slow setup part. Now we get to actually have fun using
  // these buffers. Firing off an actual I/O request involves these steps:
  // - Find a free header+payload+done buffer (in our case we only have one,
  //   so that's cool)
  // - Fill in the written-by-us part; in the block-device case, that means
  //   the request metadata header
  hdr->type = 0; // 0=read
  hdr->ioprio = 1; // prio
  hdr->sector = 0; // should be the MBR

  // - TODO: Maybe somehow "reset" the part that's not written by us? Not sure
  //   if we need, though
  *done = 17; // debugging marker, so that we can check if it worked

  // - Put the buffer into the virtqueue's "avail" array (the index-0 is actually
  //   `idx % qsz`, which wraps around after we've filled the avail array once,
  //   the value-0 is the index into the descriptor table above)
  avail->ring[0] = 0;

  // - Now, place a memory barrier so the above read is seen for sure
  __asm__ volatile ( "" : : : "memory");

  // - Now, tell the device which index into the array is the highest available one
  avail->idx = 1;

  // For reference, print out the current number of items available in the
  // "processed" part of the ring; this should be 0, since nothing has been
  // processed by the device yet.
  cor_printk("before: %x\n", used->idx);

  // - Finally, "kick" the device to tell it that it should look for something
  //   to do. We could probably skip doing this and just wait for a while;
  //   even after a kick, there's no guarantee that the request will have been
  //   processed. The actual notification about "I did a thing, please go
  //   check" will in practice be delivered to us via an interrupt. (I think)
  iowrite16(io_base+16, 0);

  // Now, in reality, we'd wait until we receive the aforementioned
  // interrupt. However, I haven't set up anything using interrupts yet. A
  // cringeworthy "alternative" is just to busy loop for a while.

  // Interestingly, this doesn't even seem to be required under QEMU/OS X.
  // Likely, the I/O write above directly triggers QEMU's virtio host driver
  // to execute the request. Obviously, this is completely unspecified
  // behvaiour we're relying on here, but let's just skip the wait while we
  // can.
  //for(int i = 0; i < 100000000; i++);
  kyield();

  // Now, magically, this index should have changed to "1" to indicate that
  // the device has processed our request buffer.
  cor_printk("after: %x\n", used->idx);
  if(used->idx != 0) {
    cor_printk("virtio call completed, ret=%u\n", *done);
    if(*done == 0) { // 0 indicate success
      // On success, the "payload" part of the buffer will contain the 512 read bytes.
      char tbuf[21];
      for(int i = 0; i < 20; i++) {
        tbuf[i] = *(char*)(payload+i);
      }
      tbuf[20] = '\0';
      if(tbuf[0] >= 'A' && tbuf[0] <= 'z') {
        cor_printk("ascii=%s\n", tbuf);
      } else {
        cor_printk("doesn't look like ascii though\n");
      }

    }
  } else {
    // this could still just be a race condition
    cor_panic("virtio call didn't complete");
  }

  // And this, dear reader, is how (surprisingly) easy it is to talk to a
  // virtio block device! Of course, this is just a spike implementation,
  // there could be buffer management, request
  // multiplexing/reordering/scheduling going on.

  cor_printk("Done initializing the virtio block device\n");

  while(1) {
    kyield();
  }
}

// When a configuration access attempts to select a device that does not exist,
// the host bridge will complete the access without error, dropping all data on
// writes and returning all ones on reads. The following code segment illustrates
// the read of a non-existent device.
uint16_t pciCheckVendor(uint8_t bus, uint8_t slot) {
  uint16_t vendor, device;
  /* try and read the first configuration register. Since there are no */
  /* vendors that == 0xFFFF, it must be a non-existent device. */
  // TODO: check all the other functions as well
  if ((vendor = pciConfigReadWord(bus,slot,0,0)) != 0xFFFF) {
    device = pciConfigReadWord(bus,slot,0,2);
    cor_printk("detected a device at %x, %x: ven=%x, dev=%x\n", bus, slot, vendor, device);

    // http://ozlabs.org/~rusty/virtio-spec/virtio-0.9.5.pdf
    if(vendor == 0x1af4 && device >= 0x1000 && device <= 0x103f) {
      uint16_t subsystem = pciConfigReadWord(bus,slot,0,0x2e);
      cor_printk("this is a virtio device with subsystem=%x\n", subsystem);
      if(subsystem==1) {
        cor_printk("this is a virtio NIC. cool\n");
      }
      if(subsystem==2) {
        setup_virtio(bus, slot, 0);
      }
    }
  }
  return vendor;
}


void pci_init(void) {
  cor_printk("Detecting PCI things...\n");
  for(int i = 0; i < 256; i++) {
    for(int j = 0; j < 256; j++) {
      pciCheckVendor(i, j);
    }
  }
}
