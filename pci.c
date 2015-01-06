#include "common.h"
#include "io.h"
#include "vendor/virtio.h"

// ref: http://wiki.osdev.org/PCI

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

  cor_printk("And here is its virtio I/O space:\n");
  for(int i = 0; i < 6; i++) {
    uint32_t state2 = sysInLong(io_base+i*4);
    cor_printk("%x: %x\n", i*4, state2);
  }

  uint32_t dflags = sysInLong(io_base);
  cor_printk("The device offers these feature bits: %x\n", dflags);

  // TODO: don't just blindly accept those
  sysOutLong(io_base+4, dflags);

  // Now for the init sequence,
  // c.f. http://ozlabs.org/~rusty/virtio-spec/virtio-0.9.5.pdf

  cor_printk("Initializing the virtio block device..\n");
  // Reset
  char state = 0;
  cor_outb(state, io_base+18);

  // Ack
  state |= VIRTIO_STATUS_ACKNOWLEDGE;
  cor_outb(state, io_base+18);

  // Drive
  state |= VIRTIO_STATUS_DRIVER;
  cor_outb(state, io_base+18);

  // Device-specific setup
  // Read feature bits
  // Discover virtqueues
  // c.f. "Virtqueue Configuration"
  // also, http://ozlabs.org/~rusty/virtio-spec/virtio-paper.pdf
  iowrite16(io_base+14, 0); // select first queue
  if(ioread16(io_base+14) != 0) {
    cor_panic("Failed to select queue0");
  }

  uint16_t qsz = ioread16(io_base+12);
  cor_printk("virtqueue has sizenum=%x\n", qsz);
  size_t rsize = vring_size(qsz, 0x1000);
  cor_printk("virtio's macros say that means a buffer size of %x\n", rsize);

  void *buf = tkalloc(rsize, "virtio vring", 0x1000); // lower align to page boundary

  for(unsigned int i = 0; i < rsize; i++) { // TODO: memzero
    *((char*)(buf+i)) = 0;
  }

  struct vring_desc *descriptors = (struct vring_desc*)buf;
  struct vring_avail *avail = buf + qsz*sizeof(struct vring_desc);
  struct vring_used *used = (struct vring_used*)ALIGN((uint64_t)avail+sizeof(struct vring_avail), 0x1000);

  cor_printk("descriptors at %p\n", descriptors);
  cor_printk("avail       at %p\n", avail);
  cor_printk("used        at %p\n", used);

  // tell the device where we placed it
  sysOutLong (io_base+8, (uint32_t)(((uint64_t)KTOP(buf)) /4096));


  // Optional MSI-X?

  // Done
  state |= VIRTIO_STATUS_DRIVER_OK;
  cor_outb(state, io_base+18);

  cor_printk("Device state set to: %x\n", state);


  // now fire off a test request
  struct virtio_blk_outhdr *hdr = (struct virtio_blk_outhdr *)tkalloc(sizeof(struct virtio_blk_outhdr), "virtio_blk request header", 0x10);
  void *payload = tkalloc(512, "virtio_blk data buffer ", 0x10);
  char *done = tkalloc(1, "virtio_blk status indicator ", 0x10);
  *done = 17; // marker

  hdr->type = 0; // 0=read
  hdr->ioprio = 1; // prio
  hdr->sector = 0; // should be the MBR

  cor_printk("Telling virtio that target is at %p\n", (uint64_t)KTOP(payload));

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

  avail->ring[0] = 0;
  __asm__ volatile ( "" : : : "memory"); // TODO: make sure this actually works
  avail->idx = 1;

  // notify
  iowrite16(io_base+16, 0);


  cor_printk("before: %x\n", used->idx);

  for(int i = 0; i < 100000000; i++);
  // ...

  cor_printk("And here is its virtio I/O space:\n");
  for(int i = 0; i < 6; i++) {
    uint32_t state2 = sysInLong(io_base+i*4);
    cor_printk("%x: %x\n", i*4, state2);
  }

  cor_printk("after: %x\n", used->idx);
  if(used->idx != 0) {
    cor_panic("SOMETHING HAPPENED");
  } else {
    cor_panic("surprisingly, nothing happened");
  }

  cor_printk("Done initializing the virtio block device\n");
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
