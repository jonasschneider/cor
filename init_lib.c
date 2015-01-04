#include "syscall.h"
#include "common.h"
#include "vendor/stdarg.h"

int exit(int ret) {
  __asm__ ( "movq %0, %%rax\n"
            "movq %1, %%rbx\n"
            "int $49"
          :
          : "r"((uint64_t)SYSCALL_EXIT), "r"((long)ret)
          : "rax", "rbx"
          );
  return 0;
}

int write(int fd, const void *buf, size_t count) {
  __asm__ ( "movq %0, %%rax\n"
            "movq %1, %%rbx\n"
            "movq %2, %%rcx\n"
            "movq %3, %%rdx\n"
            "int $49"
          :
          : "r"((uint64_t)SYSCALL_WRITE), "r"((uint64_t)fd), "r"((uint64_t)buf), "r"((uint64_t)count)
          : "rax", "rbx", "rcx", "rdx"
          );
  return 0;
}

void *moremem(size_t size) {
  void *ret;
  __asm__ ( "movq %1, %%rax\n"
            "movq %2, %%rbx\n"
            "int $49\n"
            "movq %%rax, %0\n"
          : "=r"(ret)
          : "r"((uint64_t)SYSCALL_MOREMEM), "r"((uint64_t)size)
          : "rax", "rbx"
          );
  return ret;
}

void *malloc(size_t size) {
  return moremem(size);
}

size_t strlen(const char *str) {
  size_t i = 0;
  while(*str) {
    i++;
    str++;
  }
  return i;
}

#define WRITEC_BUF 256
static char writec_buf[256];
static int writec_buf_i = 0;
void writec(char c) {
  if(writec_buf_i < WRITEC_BUF) {
    writec_buf[writec_buf_i++] = c;
  }
}

int _printf_print(const char *str) {
  while(*str) {
    writec(*str);
    str++;
  }
  return 0;
}

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
    _printf_print("ERR"); // at least indicate the error
  buffer[i] = alphabet[(value % base)];

  _printf_print(buffer+i);
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



int vprintf(const char *format, va_list ap) {
  while(*format) {
    short islong = 0;
    if(*format == '%') {
      format++;
      if(*format == '%') {
        writec('%');
        continue;
      }
      if(*format == 'l') {
        islong = 1;
        format++;
      }
      if(*format == 'p') {
        _printf_print("0x");
        printk_lhex(va_arg(ap, unsigned long));
      } else if(*format == 'u') {
        printk_uint(va_arg(ap, uint));
      } else if(*format == 'x') {
        // another x allows you to skip the "0x" part
        if(*(format+1) == 'x') {
          format++;
        } else {
          _printf_print("0x");
        }

        if(islong) {
          printk_lhex(va_arg(ap, unsigned long));
        } else {
          printk_hex(va_arg(ap, uint));
        }
      } else if(*format == 's') {
        char *s = va_arg(ap, char*);
        while(*s) {
          writec(*s);
          s++;
        }
      } else {
        _printf_print("[_printf: invalid format]");
      }
    } else {
      writec(*format);
    }
    format++;
  }

  return 0;
}

int printf(const char *format, ...) {
  writec_buf_i = 0;

  va_list ap;
  va_start(ap, format); //Requires the last fixed parameter (to get the address)

  vprintf(format, ap);

  va_end(ap);
  write(1, writec_buf, writec_buf_i);
  return 0;
}

void main();

// TODO: .data and .bss sections break (probably anything besides .text)
void _start() {
  printf("Hello from _start.\n");
  main();
  exit(0xBABE);

  while(1);
}
