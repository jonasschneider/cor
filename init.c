#include "syscall.h"
#include "common.h"

// TODO: .data and .bss sections break (probably anything besides .text)
void _start() {
  char *str = "Hello, world from init.\nI live at %p, and cor_printk is at %p.\n";
  __asm__ ( "movq %0, %%rax\n"
            "movq %1, %%rbx\n"
            "int $49"
          :
          : "r"((uint64_t)0x1337), "r"(str)
          : "rax", "rbx"
          );
  while(1);

  //cor_printk(str, _start, cor_printk);
}
