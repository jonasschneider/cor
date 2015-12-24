#include "cor/syscall.h"
.globl   dummy_isr
.align   4

.globl irq_log
irq_log:
  .fill 256, 1, 0

is_return_from_trampoline:
  pop %rax # restore original rax
  cli # TODO
  jmp trampoline_from_user

isr_dispatcher:
  # no==49 (syscall): return from userspace
  sub $49, %rax
  jz is_return_from_trampoline
  add $49, %rax

  # set the appropriate flag in irq_log
  push %rbx
  mov %rax, %rbx
  movabs $irq_log, %rax
  movb $1, (%rax, %rbx)
  mov %rbx, %rax
  pop %rbx


  // TODO: only push/pop the x86-64 caller-saved registers here
  push %rbx
  push %rcx
  push %rdx
  push %r8
  push %r9
  push %r10
  push %r11
  push %rdi

  # Place the IRQ number parameter correctly:
  # The first parameters are placed in the %rdi, %rsi, %rdx, %rcx, %r8, %r8 registers.
  # More parameters go on the stack.
  # We only need a single parameter, the IRQ number, which will go into %rdi.
  # However, in Rust, this parameter is an u8, so Rust will read it back from the
  # %dil register, which means the lowest 8 bytes of %rdi. We might be able to optimize this a bit further.
  mov %rax, %rdi

  call handle_irq

  pop %rdi
  pop %r11
  pop %r10
  pop %r9
  pop %r8
  pop %rdx
  pop %rcx
  pop %rbx
  pop %rax

  iretq

.global timer_isr
timer_isr:
  push %rax

  ; # increment our timer
  ; mov 0x81000|0x0000008000000000, %rax
  ; inc %rax
  ; mov %rax, 0x81000|0x0000008000000000

  ; # tell the PIC that we've handled the interrupt and it can send the next one
  ; # (see http://wiki.osdev.org/8259_PIC#End_of_Interrupt)
  ; # TODO: Isn't this a race condition? Do we need to cli here first and then sti
  ; # before the iretq?
  ; mov $0x20, %al # command port of PIC1
  ; outb %al, $0x20

  pop %rax
  iretq

.global asm_eoi
asm_eoi:
  # set EOI
  mov $0x20, %al # clear flag?
  outb %al, $0x20 # command port of PIC1
  outb %al, $0xa0 # command port of PIC2
  ret

#include "intstubs.s~"
