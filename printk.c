#include "vendor/stdarg.h"
#include "common.h"

void (*cor_current_writec)(const char c);

static void print_itoa(unsigned long value, const uint base, const char *alphabet) {
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
  print_itoa((unsigned long)value, 10, "0123456789");
}

static void printk_hex(uint value) {
  print_itoa((unsigned long)value, 16, "0123456789abcdef");
}


static void printk_lhex(unsigned long value) {
  print_itoa(value, 16, "0123456789abcdef");
}


int cor_printk(const char *format, ...) {
  va_list ap;
  va_start(ap, format); //Requires the last fixed parameter (to get the address)

  while(*format) {
    short islong = 0;
    if(*format == '%') {
      format++;
      if(*format == '%') {
        (*cor_current_writec)('%');
        continue;
      }
      if(*format == 'l') {
        islong = 1;
        format++;
      }
      if(*format == 'p') {
        cor_printk("0x");
        printk_lhex(va_arg(ap, unsigned long));
      } else if(*format == 'u') {
        printk_uint(va_arg(ap, uint));
      } else if(*format == 'x') {
        // another x allows you to skip the "0x" part
        if(*(format+1) == 'x') {
          format++;
        } else {
          cor_printk("0x");
        }

        if(islong) {
          printk_lhex(va_arg(ap, unsigned long));
        } else {
          printk_hex(va_arg(ap, uint));
        }
      } else if(*format == 's') {
        char *s = va_arg(ap, char*);
        while(*s) {
          (*cor_current_writec)(*s);
          s++;
        }
      } else {
        cor_printk("[cor_printk: invalid format]");
      }
    } else {
      (*cor_current_writec)(*format);
    }
    format++;
  }

  va_end(ap);
  return 0;
}
