#include "syscall.h"
#include "common.h"

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
          : "rax", "rbx"
          );
  return 0;
}

size_t strlen(const char *str) {
  size_t i = 0;
  while(*str) {
    i++;
    str++;
  }
  return i;
}

int printf(const char *fmt, ...) {
  return write(1, fmt, strlen(fmt));
}

void main();

// TODO: .data and .bss sections break (probably anything besides .text)
void _start() {
  main();
  exit(0xBABE);

  while(1);
}
