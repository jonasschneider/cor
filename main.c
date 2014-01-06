#include <stdarg.h>
int my_kernel_subroutine() {
  return 0xbeef;
}

int lawl = 0xdeadbeef;

void console_clear(void);
int printk(const char *text, ...);

void kernel_main(void) {
  console_clear();
  printk("Hello from the kernel.\nYes, we can multiline.\n");
  printk("Leet is \'%u\'\n", 1337);
  printk("Haxx is \'%x\'\n", 0x1234321);

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
  if(console_col == console_width-1 || c == '\n') {
    console_line++;
    console_col = 0;
    if(c == '\n') return;
  }

  int grid_index = (console_col++) + console_line*console_width;

  *((unsigned char*)0xB8000+grid_index*2) = c;
}

void printk_uint(unsigned int value) {
  const int max_digits = 20;
  char buffer[max_digits+1]; // FIXME: yeah, this is never going to break, ever
  buffer[max_digits] = '\0';
  int i = max_digits-1;
  char overflow = 0;
  while(value > 10) {
    if(i == 0) { // leave one for the last digit
      overflow = 1;
    } else {
      buffer[i--] = 0x30 + (value % 10);
    }
    value /= 10;
  }
  if(overflow)
    printk("ERR"); // at least indicate the error
  buffer[i--] = 0x30 + value;

  printk(buffer+i+1);
}

void printk_hex(unsigned int value) {
  printk("0x");
  unsigned int trailing_zeroes = 0;
  for(unsigned int i = sizeof(value) * 2; i > 0; i--) {
    char nibble = (value & (0xf << (sizeof(value)*2-4*i+4) )) >> (sizeof(value)*2-4*i+4);
    if(nibble != 0) {
      trailing_zeroes = i;
      break;
    }
  }
  printk("trailing:%u  ", trailing_zeroes);
  for(unsigned int i = 0; i < sizeof(value)*2; i++) {
    char nibble = (value & (0xf << (sizeof(value)*2-4*i+4) )) >> (sizeof(value)*2-4*i+4);
    printk("%u", (unsigned int)nibble);
    //writec_k("0123456789abcdef"[(unsigned int)nibble]);
  }
}

int printk(const char *format, ...) {
  va_list ap;
  va_start(ap, format); //Requires the last fixed parameter (to get the address)

  while(*format) {
    if(*format == '%') {
      format++;
      if(*format == '%') {
        writec_k('%');
      } else if(*format == 'u') {
        printk_uint(va_arg(ap, unsigned int));
      } else if(*format == 'x') {
        printk_hex(va_arg(ap, unsigned int));
      }
    } else {
      writec_k(*format);
    }
    format++;
  }

  va_end(ap);
  return 0;
}
