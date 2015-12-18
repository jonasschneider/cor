#include "common.h"
#include "io.h"

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
  cor_printk("Virtio IRQ is %x, IO base location %x\n", irq, io_base);

  // This is pretty much all we actually interface with PCI; once we have the
  // I/O base port, we're golden.

  cor_printk("Letting Rustland set up the Virtio device..\n");
  //virtio_init(io_base);
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
