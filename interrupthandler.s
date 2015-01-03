.globl   dummy_isr
.align   4

dummy_isr:
  push %rax
  push %rbx
  push %rcx
  push %rdx
  push %r8
  push %r9
  push %r10
  push %r11

  mov %rax, %rdi
  call cor_syscall

  pop %r11
  pop %r10
  pop %r9
  pop %r8
  pop %rdx
  pop %rcx
  pop %rbx
  pop %rax

  iretq
