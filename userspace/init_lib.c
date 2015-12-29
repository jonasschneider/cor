#include <cor/syscall.h>
#include <vendor/stdarg.h>
#include <stdint.h>

int exit(int ret) {
  __asm__ ( "movq %0, %%rax\n"
            "movq %1, %%rbx\n"
            "int $49"
          :
          : "r"((uint64_t)SYSCALL_EXIT), "r"((uint64_t)ret)
          : "rax", "rbx", "rcx", "rdx", "r12", "r13", "r14", "r15"
          );
  return 0;
}

int open(const char *path, int flags) {
  __asm__ ( "movq %0, %%rax\n"
            "movq %1, %%rbx\n"
            "movq %2, %%rcx\n"
            "int $49"
          :
          : "r"((uint64_t)SYSCALL_OPEN), "r"((uint64_t)path), "r"((uint64_t)flags)
          : "rax", "rbx", "rcx", "rdx", "r12", "r13", "r14", "r15"
          );
  return 0;
}

int read(int fd, const void *buf, size_t count) {
  __asm__ ( "movq %0, %%rax\n"
            "movq %1, %%rbx\n"
            "movq %2, %%rcx\n"
            "movq %3, %%rdx\n"
            "int $49"
          :
          : "r"((uint64_t)SYSCALL_READ), "r"((uint64_t)fd), "r"((uint64_t)buf), "r"((uint64_t)count)
          : "rax", "rbx", "rcx", "rdx", "r12", "r13", "r14", "r15"
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
          : "rax", "rbx", "rcx", "rdx", "r12", "r13", "r14", "r15"
          );
  return 0;
}

void *mallocstart = (void*)0x50000;

void *malloc(size_t size) {
  mallocstart += size;
  return mallocstart - size;
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

static void print_itoa(unsigned long value, const unsigned int base, const char *alphabet) {
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

static void printk_uint(unsigned int value) {
  print_itoa((unsigned long)value, 10, "0123456789");
}

static void printk_hex(unsigned int value) {
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
        printk_uint(va_arg(ap, unsigned int));
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
          printk_hex(va_arg(ap, unsigned int));
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

#define stub(n) void n() {printf("%s\n", #n);while(1) {} }

int stdin = 0;
int stderr = 1;
int stdout = 2;

int *__errno_location;

stub(abort);
stub(closedir);
stub(execve);
stub(fork);
stub(getenv);
stub(getpgrp);
stub(getpid);
stub(init);
stub(initshellproc);
stub(ioctl);
stub(kill);
stub(killpg);
stub(_longjmp);
stub(raise);
stub(readdir);
stub(reset);
stub(_setjmp);
stub(setpgrp);
stub(sigsetmask);
stub(wait3);
stub(execl);
stub(pipe);
stub(fcntl);
stub(realloc);
stub(chdir);
stub(strcat);
stub(readlink);
stub(lstat);
stub(free);
stub(signal);
stub(atoi);
stub(opendir);
stub(_exit);
stub(strcpy);
stub(fgets);
stub(putc);
stub(fopen);
stub(strcmp);
stub(putchar);
stub(fwrite);
stub(atol);
stub(sprintf);
stub(stat);
stub(puts);
stub(geteuid);
stub(getegid);
stub(fprintf);
stub(close);
stub(isatty);
stub(fputs);
stub(umask);
