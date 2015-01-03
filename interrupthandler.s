#include "syscall.h"
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

  # Reorder paramters:
  # According to the ABI, the first 6 integer or pointer arguments to a function are
  # passed in registers. The first is placed in rdi, the second in rsi, the third in rdx,
  # and then rcx, r8 and r9. Only the 7th argument and onwards are passed on the stack.
  mov %rbx, %rdi
  mov %rcx, %rsi

  cmp $SYSCALL_EXIT, %rax
  jne bye
  call syscall_exit

bye:
  pop %r11
  pop %r10
  pop %r9
  pop %r8
  pop %rdx
  pop %rcx
  pop %rbx
  pop %rax

  iretq
