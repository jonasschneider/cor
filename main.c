#include "common.h"
#include "printk.h"
#include "chrdev_serial.h"
#include "elf.h"

int my_kernel_subroutine() {
  return 0xbeef;
}

extern char cor_stage2_init_data;
extern int cor_stage2_init_data_len;

int lawl = 0xdeadbeef;

void console_clear(void);

void cor_panic() {
  cor_printk("Panicking.\n");
  while(1) {};
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

int dummy_isr;
void testisr();

void cor_1bitpanic() {
  cor_printk("FIRED!\n");
  cor_panic();
}

void cor_writec(const char c);
void (*cor_current_writec)(const char c) = cor_writec;

#pragma pack(push, 1)
struct {
  uint16_t limit;
  uint64_t base;
} idtr;
#pragma pack(pop)

void kernel_main(void) {
  if(sizeof(uint8_t) != 1) {
    cor_printk("sizeof(uint8_t) = %d !!", sizeof(uint8_t));
    cor_panic();
  }
  if(sizeof(uint16_t) != 2) {
    cor_printk("sizeof(uint16_t) = %d !!", sizeof(uint16_t));
    cor_panic();
  }
  if(sizeof(uint32_t) != 4) {
    cor_printk("sizeof(uint32_t) = %d !!", sizeof(uint32_t));
    cor_panic();
  }
  if(sizeof(uint64_t) != 8) {
    cor_printk("sizeof(uint64_t) = %d !!", sizeof(uint64_t));
    cor_panic();
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

  cor_printk("Initializing interrupts..");

  cor_printk("Calling testisr at %p\n",testisr);

  testisr();

  // cf. intel_64_software_developers_manual.pdf pg. 1832
  void *target = (void*)&dummy_isr;

  void *base = (void*)0x6000;
  const int entrysize = 16; // in bytes
  const int n_entry = 40;
  idtr.base = (uint64_t)base;
  idtr.limit = entrysize * n_entry;

  for(int i = 0; i < n_entry; i++) {
    void *offset = base+(i*entrysize);

    *(uint16_t*)(offset+0) = (uint16_t) ((uint64_t)target >> 0);
    *(uint16_t*)(offset+2) = (uint16_t) 1; // segment
    *(uint8_t*)(offset+4) = (uint8_t) 0; // zero
    *(uint8_t*)(offset+5) = (uint8_t) 1<<7; // flags
    *(uint16_t*)(offset+6) = (uint16_t) ((uint64_t)target >> 16);
    *(uint32_t*)(offset+8) = (uint32_t) ((uint64_t)target >> 32);
    *(uint32_t*)(offset+12) = (uint32_t) 0; // reserved
  }


  void *x = (void*)&idtr;

  __asm__ (
    "lidt (%0)"
    : : "p" (x)
  );

  cor_printk("done.\n");

  __asm__ (
    "sti"
  );

  cor_printk("enabled interrupts again. firing one..\n");
  __asm__ ( "int $35" );
  __asm__ ( "hlt" );

  cor_printk("Exec'ing init.\n");

  //cor_dump_page_table((uint64_t *)0x1000, 1);
  cor_elf_exec(&cor_stage2_init_data, cor_stage2_init_data_len);

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
