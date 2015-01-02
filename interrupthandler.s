.globl   dummy_isr
.align   4

dummy_isr:
  push %rax
  push %rcx
  push %rdx
  push %r8
  push %r9
  push %r10
  push %r11

  call cor_1bitpanic

  push %r11
  push %r10
  push %r9
  push %r8
  push %rdx
  push %rcx
  push %rax

  iretq
