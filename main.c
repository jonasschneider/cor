int my_kernel_subroutine() {
  return 0xbeef;
}

int lawl = 0xdeadbeef;

void console_clear(void);
int printk(const char *text);

void kernel_main(void) {
  console_clear();
  printk("Hello from the kernel.\nYes, we can multiline.\n");

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

void writec_k(const char c) {
  if(console_col == console_height-1 || c == '\n') {
    console_line++;
    console_col = 0;
    if(c == '\n') return;
  }

  int grid_index = (console_col++) + console_line*console_width;

  *((unsigned char*)0xB8000+grid_index*2) = c;
}

int printk(const char *text) {
  while(*text) {
    writec_k(*text);
    text++;
  }
  return 0;
}
