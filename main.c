#include <stdint.h>
#include "common.h"
#include "printk.h"
#include "chrdev_serial.h"

int my_kernel_subroutine() {
  return 0xbeef;
}

int lawl = 0xdeadbeef;

void console_clear(void);

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

void cor_writec(const char c);
void (*cor_current_writec)(const char c) = cor_writec;

void kernel_main(void) {
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
  //cor_dump_page_table((uint64_t *)0x1000, 1);

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
