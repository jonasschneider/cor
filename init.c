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

// TODO: .data and .bss sections break (probably anything besides .text)
void _start() {
  char *str = "Hello, world from userland!\n";
  write(1, str, 23);
  exit(0xBABE);

  while(1);
}
