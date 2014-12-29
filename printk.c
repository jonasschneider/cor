#include <stdarg.h>
#include "common.h"
#include "printk.h"

void (*cor_current_writec)(const char c);

static void print_itoa(uint value, const uint base, const char *alphabet) {
  const int max_digits = 20;
  char buffer[max_digits+1]; // FIXME: yeah, this is never going to break, ever
  buffer[max_digits] = '\0';
  int i = max_digits-1;
  char overflow = 0;
  while(value >= base) {
    if(i == 0) { // leave one for the last digit
      overflow = 1;
    } else {
      buffer[i--] = alphabet[(value % base)];
    }
    value /= base;
  }
  if(overflow)
    cor_printk("ERR"); // at least indicate the error
  buffer[i] = alphabet[(value % base)];

  cor_printk(buffer+i);
}

static void printk_uint(uint value) {
  print_itoa(value, 10, "0123456789");
}

static void printk_hex(uint value) {
  print_itoa(value, 16, "0123456789abcdef");
}

int cor_printk(const char *format, ...) {
  va_list ap;
  va_start(ap, format); //Requires the last fixed parameter (to get the address)

  while(*format) {
    if(*format == '%') {
      format++;
      if(*format == '%') {
        (*cor_current_writec)('%');
      } else if(*format == 'u') {
        printk_uint(va_arg(ap, uint));
      } else if(*format == 'x') {
        // another x allows you to skip the "0x" part
        if(*(format+1) == 'x') {
          format++;
        } else {
          cor_printk("0x");
        }
        printk_hex(va_arg(ap, uint));
      }
    } else {
      (*cor_current_writec)(*format);
    }
    format++;
  }

  va_end(ap);
  return 0;
}
