#include <stdarg.h>

typedef unsigned int uint;

int my_kernel_subroutine() {
  return 0xbeef;
}

int lawl = 0xdeadbeef;

void console_clear(void);
int printk(const char *text, ...);

void kernel_main(void) {
  console_clear();
  printk("Hello from the kernel.\nYes, we can multiline.\n");
  printk("Leet is \'%u\', 0 is \'%u\'\n", 1337, 0);
  printk("Haxx is \'%x\', 0 is \'%x\'\n", 0x1234321, 0);

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

void print_itoa(uint value, const uint base, const char *alphabet) {
  const int max_digits = 20;
  char buffer[max_digits+1]; // FIXME: yeah, this is never going to break, ever
  buffer[max_digits] = '\0';
  int i = max_digits-1;
  char overflow = 0;
  while(value > base) {
    if(i == 0) { // leave one for the last digit
      overflow = 1;
    } else {
      buffer[i--] = alphabet[(value % base)];
    }
    value /= base;
  }
  if(overflow)
    printk("ERR"); // at least indicate the error
  buffer[i] = alphabet[(value % base)];

  printk(buffer+i);
}

void printk_uint(uint value) {
  print_itoa(value, 10, "0123456789");
}

void printk_hex(uint value) {
  printk("0x");
  print_itoa(value, 16, "0123456789abcdef");
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
        printk_uint(va_arg(ap, uint));
      } else if(*format == 'x') {
        printk_hex(va_arg(ap, uint));
      }
    } else {
      writec_k(*format);
    }
    format++;
  }

  va_end(ap);
  return 0;
}
