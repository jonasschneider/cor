#include "cor/syscall.h"
.globl   dummy_isr
.align   4

is_return_from_trampoline:
  pop %rax
  cli
  jmp trampoline_from_user

isr_dispatcher:
  sub $49, %rax
  jz is_return_from_trampoline
  pop %rax

  #push %rax # TODO: figure out why this breaks everything
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
  mov 7*8(%rsp), %rdi #interrupt no

  # TODO: should probably optimize these to only do this lifting on syscall interrupts, cycles everywhea!
  mov %rax, %rsi # arg1 (if syscall)
  mov %rdx, %r8 # arg4 (if syscall)
  mov %rbx, %rdx # arg2 (if syscall)
  mov %rcx, %rcx # arg3 (if syscall)

  call interrupt

  pop %r11
  pop %r10
  pop %r9
  pop %r8
  pop %rdx
  pop %rcx
  pop %rbx
  #pop %rax

  iretq

.global timer_isr
timer_isr:
  push %rax

  # increment our timer
  mov 0x81000|0x0000008000000000, %rax
  inc %rax
  mov %rax, 0x81000|0x0000008000000000

  # tell the PIC that we've handled the interrupt and it can send the next one
  # (see http://wiki.osdev.org/8259_PIC#End_of_Interrupt)
  # TODO: Isn't this a race condition? Do we need to cli here first and then sti
  # before the iretq?
  mov $0x20, %al # command port of PIC1
  outb %al, $0x20

  pop %rax
  iretq

#include "intstubs.s~"
