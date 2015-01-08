#include "cor/syscall.h"
.globl   dummy_isr
.align   4

dummy_isr:
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
  #mov %rdx, %rdx

  cmp $SYSCALL_WRITE, %rax
  jne check_exit
  call syscall_write

check_exit:
  cmp $SYSCALL_EXIT, %rax
  jne check_moremem
  call syscall_exit

check_moremem:
  cmp $SYSCALL_MOREMEM, %rax
  jne bye
  call syscall_moremem

bye:
  pop %r11
  pop %r10
  pop %r9
  pop %r8
  pop %rdx
  pop %rcx
  pop %rbx

  iretq

.global timer_isr
timer_isr:
  push %rax
  push %rdx
  # increment our timer
  mov 0x80000|0x0000008000000000, %eax
  inc %eax
  mov %eax, 0x80000|0x0000008000000000

  call cor_hitmarker

  # re-enable interrupt
  mov $0x20, %al # EOI command
  mov $0x20, %dx # command port of PIC1
  outb %al, $0x20

  pop %rdx
  pop %rax
  iretq
