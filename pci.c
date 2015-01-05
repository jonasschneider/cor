#include "common.h"
#include "io.h"

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

unsigned char lolin(unsigned short int port) ;
unsigned char lolout(unsigned char value, unsigned short int port);

// When a configuration access attempts to select a device that does not exist,
// the host bridge will complete the access without error, dropping all data on
// writes and returning all ones on reads. The following code segment illustrates
// the read of a non-existent device.
uint16_t pciCheckVendor(uint8_t bus, uint8_t slot) {
  uint16_t vendor, device;
  /* try and read the first configuration register. Since there are no */
  /* vendors that == 0xFFFF, it must be a non-existent device. */
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
        cor_printk("This is a virtio block device!\nThis is its configuration space:\n");
        for(int i = 0; i < 0x3c; i+=4) {
          cor_printk("%x = %x\n", i, pciConfigReadLong(bus, slot, 0, i));
        }

        // BAR0 is an I/O region
        uint32_t bar0 = pciConfigReadLong(bus, slot, 0, 0x10);
        uint16_t io_base = bar0 & 0xFFFFFFFC;

        /*// BAR1 is a memory region; looks like we don't need it
        uint32_t bar1 = pciConfigReadLong(bus, slot, 0, 0x14);
        void *base = (void*)((ptr_t)bar1 & 0xFFFFFFF0);

        // "To determine the amount of address space needed by a PCI device,
        // you must save the original value of the BAR, write a value of all
        // 1's to the register, then read it back. The amount of memory can
        // then be determined by masking the information bits, performing a
        // bitwise NOT ('~' in C), and incrementing the value by 1."
        pciConfigWriteLong(bus, slot, 0, 0x14, 0xFFFFFFFF);
        uint32_t modbar = pciConfigReadLong(bus, slot, 0, 0x14);
        pciConfigWriteLong(bus, slot, 0, 0x14, bar1);

        ioread8(vp_dev->ioaddr + VIRTIO_PCI_STATUS)

        uint32_t memsize = ~(modbar&0xFFFFFFF0) + 1;
        cor_printk("device shows us a used memory size of %x\n", memsize);

        cor_printk("dumping memory region starting at %p\n", base);
        for(int i = 0; i < 10; i++) {
          uint64_t *addr = (uint64_t*)base + i;
          cor_printk("%p = %lx\n", addr, *addr);
        } */

        cor_printk("And here is its virtio I/O space:\n");
        for(int i = 0; i < 15; i++) {
          uint32_t state2 = sysInLong(io_base+i*4);
          cor_printk("%x: %x\n", i*4, state2);
        }
        cor_printk("Now by asm:\n");
        for(int i = 0; i < 20; i++) {
          uint32_t state2 = (uint32_t)lolin(io_base+i);
          cor_printk("%x: %x\n", i, state2);
        }
        cor_printk("Now by cor_inb:\n");
        for(int i = 0; i < 20; i++) {
          uint32_t state2 = (uint32_t)cor_inb(io_base+i);
          cor_printk("%x: %x\n", i, state2);
        }

        uint32_t dflags = sysInLong(io_base);
        cor_printk("virtio has device flags: %x\n", dflags);

        char state = cor_inb(io_base+18);
        cor_printk("       virtio has state: %u, resetting...\n", state);

        cor_outb(0, io_base+18);

        state = cor_inb(io_base+18);
        cor_printk("       virtio now state: %u\n", state);

        cor_printk("config space again:\n");
        for(int i = 0; i < 0x3c; i+=4) {
          cor_printk("%x = %x\n", i, pciConfigReadLong(bus, slot, 0, i));
        }
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
