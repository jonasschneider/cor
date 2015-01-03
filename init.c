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

// TODO: .data and .bss sections break (probably anything besides .text)
void _start() {
  exit(0xBABE);
  //char *str = "Hello, world from init.\nI live at %p, and cor_printk is at %p.\n";

  while(1);

  //cor_printk(str, _start, cor_printk);
}
