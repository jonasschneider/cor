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

.globl   testisr
.globl   before
testisr:
setidtr:  lidt idtr
  movq $dummy_isr, %rax
  mov %ax, 0x6000+49*16
  movw $0x8, 0x6000+49*16+2 # segment
  movw $0x8e00, 0x6000+49*16+4
  shr $16, %rax
  mov %ax, 0x6000+49*16+6
  shr $16, %rax
  movw %rax, 0x6000+49*16+8

setsp: mov $0x70000, %rsp
  #sti

pushy: pushq $0x1234

before: int $49
  hlt

int_handler:
  call cor_1bitpanic
  hlt

idtr:
  .short (52*16)-1
  .quad 0x6000
