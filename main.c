#include "common.h"
#include "printk.h"

int my_kernel_subroutine() {
  return 0xbeef;
}

int lawl = 0xdeadbeef;

void console_clear(void);

void kernel_main(void) {
  console_clear();
  cor_printk("Hello from the kernel.\nYes, we can multiline.\n");
  cor_printk("Leet is \'%u\', 0 is \'%u\'\n", 1337, 0);
  cor_printk("Haxx is \'%x\', 0 is \'%x\'\n", 0x1234321, 0);

  while(1) {}

  {
    int i = 0;
    while(1) {
      *((unsigned char*)0xB8000+(i++)) = 'X';
    }
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
