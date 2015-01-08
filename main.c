#include "common.h"
#include "chrdev_serial.h"
#include "elf.h"
#include "tss.h"
#include "mm.h"
#include "pci.h"
#include "pic.h"
#include "timer.h"
#include "interrupt.h"
#include "sched.h"

extern char cor_stage2_init_data;
extern int cor_stage2_init_data_len;

int lawl = 0xdeadbeef;

void console_clear(void);

unsigned long *timer = (unsigned long*)(0x81000|0x0000008000000000);

void cor_panic(const char *msg) {
  cor_printk("\nPANIC: %s\n    (at t=%x)\n", msg, *timer);
  // panicking is actually pretty hard; we should clean up and disable as many interrupts
  // etc. as possible here, since these can wake us up from HLT.
  // TODO: Unclear behaviour if we panic inside an interrupt handler (like, within a syscall)
  // It looks like the timer doesn't interrupt us here, maybe because it's low-priority?
  while(1) {
    __asm__ ( "hlt" );
  }
}

void cor_dump_page_table(uint64_t *start, int level) {
  for(uint64_t *page_table_entry = start; page_table_entry < start+512; page_table_entry++) {
    if(*page_table_entry != 0) {
      uint64_t *base = (uint64_t *)(*page_table_entry & ~((1<<12)-1));
      if(level == 1) {
        cor_printk("%x: base %x bits ", (uint64_t)page_table_entry, base);
        if(*page_table_entry & (1<<5)) { cor_printk("A"); } else { cor_printk("-"); } // accessed
        if(*page_table_entry & (1<<1)) { cor_printk("W"); } else { cor_printk("-"); } // writable
        if(*page_table_entry & (1<<0)) { cor_printk("P"); } else { cor_printk("-"); } // present
        cor_printk("\n");
      } else {
        if(level < 4)
          cor_dump_page_table(base, level+1);
      }
    }
  }
}



void cor_hitmarker() {
  cor_printk("FIRED! timer=%x\n", *timer);
}

uint64_t syscall_write(uint64_t fd64, uint64_t buf64, size_t count64) {
  // TODO: make sure these are harmless
  int fd = (int)fd64;
  const void *buf = (const void *)buf64;
  size_t count = (size_t)count64;

  cor_printk("write() fd=%x, buf=%p, n=%x:\n", fd, buf, count);
  cor_printk("    | ");
  for(size_t i = 0; i < count; i++) {
    char c = *((char*)buf+i);
    putc(c);
    if(c == '\n' && i < (count-1)) {
      cor_printk("    | ");
    }
  }
  return 0;
}

void syscall_exit(uint64_t ret64) {
  int ret = (int)ret64;
  cor_printk("exit() ret=%x\n", ret);
  cor_panic("init exited");
}

void cor_writec(const char c);
void (*cor_current_writec)(const char c) = cor_writec;

void kernel_main(void) {
  if(sizeof(uint8_t) != 1) {
    cor_printk("sizeof(uint8_t) = %d !!", sizeof(uint8_t));
    cor_panic("assertion failure");
  }
  if(sizeof(uint16_t) != 2) {
    cor_printk("sizeof(uint16_t) = %d !!", sizeof(uint16_t));
    cor_panic("assertion failure");
  }
  if(sizeof(uint32_t) != 4) {
    cor_printk("sizeof(uint32_t) = %d !!", sizeof(uint32_t));
    cor_panic("assertion failure");
  }
  if(sizeof(uint64_t) != 8) {
    cor_printk("sizeof(uint64_t) = %d !!", sizeof(uint64_t));
    cor_panic("assertion failure");
  }

  console_clear();
  uint32_t revision = *((uint32_t*)0x6fffa);
  cor_printk("\n   Cor rev. %xx\n\n",revision);
  cor_printk("Hello from the kernel.\nYes, we can multiline.\n");
  cor_printk("Leet is \'%u\', 0 is \'%u\'\n", 1337, 0);
  cor_printk("Haxx is \'%x\', 0 is \'%x\'\n", 0x2003, 0);

  cor_chrdev_serial_init();
  cor_printk("Switching to serial...\n");
  cor_current_writec = cor_chrdev_serial_write;
  cor_printk("Switched to serial console.\n");

  // TODO: document this crazy rust stack-fixing thing
  uint64_t res = 1;
  __asm__ (
    "mov %0, %%fs:0x70"
    : : "a" (res)
    );
  __asm__ (
    "mov %%fs:0x70, %0"
    : "=a" (res)
    );
  cor_printk("XXX fs:70=%x\n",res);

  cor_printk("Initializing MM.. ");
  mm_init();
  cor_printk("OK.\n");

  // The first half of the interrupt setup is the software side. The CPU has
  // an INTR pin. If that goes high, it knows that (some kind of) interrupt
  // has occured. It then asks on some special I/O lines /which/ kind of
  // interrupt has actually occured. We'll care about this in a sec, right
  // now, we'll set up our half of the work: After knowing which interrupt
  // occured, the CPU looks in the interrupt descriptor table (IDT) to find
  // out how to handle the interrupt. And this table is exactly what we define
  // here.
  cor_printk("Initializing interrupts.. ");
  interrupt_init();

  // Okay, this was the software side. Now, the actually useful part of
  // interrupts is being able to receive them from the outside world, i.e. the
  // chipset and devices attached to it. As mentioned above, the CPU has this
  // INTR pin for receiving these from the outside world. However, you can
  // only attach one device to that one pin, which sucks. So, someone decided
  // to put a kind of multiplexer in front of the INTR line. This is exactly
  // what the Programmable Interrupt Controller (PIC) is. It can be attached
  // to, like, 16 devices that can then each generate interrupts, and the PIC
  // will inform the CPU about them and also tell it *which* one of the devices
  // triggered it.

  // To initialize the PIC, we need to tell it which kinds of interrupts to
  // generate; the CPU knows 256 types, and the PIC setup on the board allows
  // 16 types to be generated by hardware. The first 32 (0..31) are reserved
  // for CPU exceptions, so we'll map the PIC-initiated interrupts to
  // 0x20..0x3f.
  pic_init(0x20);
  cor_printk("OK.\n");

  // Now we can actually receive interrupts from hardware on our chipset! The
  // first thing we'll do is set up our timer, which is an external oscillator
  // that occasionally (predictibly) fires interrupts so we know that time has
  // passed.
  *timer = 0;
  cor_printk("Starting timer.. ");
  timer_init();
  cor_printk("OK.\n");

  cor_printk("Setting up TSS.. ");
  tss_setup();
  cor_printk("OK.\n");

  cor_printk("Initializing PCI.. ");
  pci_init();
  cor_printk("OK.\n");

  cor_printk("Setting up scheduler.. ");
  //sched_init();
  cor_printk("OK.\n");

  cor_printk("Exec'ing init.\n");

  //cor_dump_page_table((uint64_t *)0x1000, 1);
  cor_elf_exec(&cor_stage2_init_data, cor_stage2_init_data_len);


  cor_printk("reached the unreachable");
  while(1) {
    cor_printk(".");
    for(int i = 0; i < 1000000; i++);
  }
}

const int console_width = 80;
const int console_height = 25;
int console_line = 0;
int console_col = 0;

void console_clear(void) {
  console_line = 0;
  console_col = 0;
  for(int i = 0; i < (console_width*console_height); i++)
    *((unsigned char*)0xB8000+i*2) = ' ';
}

void cor_writec(const char c) {
  if(console_col == console_width-1 || c == '\n') {
    console_line++;
    console_col = 0;
    if(c == '\n') return;
  }

  int grid_index = (console_col++) + console_line*console_width;

  *((unsigned char*)0xB8000+grid_index*2) = c;
}
