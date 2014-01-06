int my_kernel_subroutine() {
  return 0xbeef;
}

int lawl = 0xdeadbeef;

int printk(const char *text);

void kernel_main(void) {
  *((unsigned char*)0xB8000) = 'X';
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

inline void writec_k(const char c) {
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
  //while(1) {
    writec_k(*text);
    text++;
  }
  return 0;
}
