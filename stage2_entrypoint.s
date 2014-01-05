.code64

.section entrypoint
.globl  _start
_start:
  mov $kernel_main, %rax
  jmp *%rax
