.code16gcc
.globl  _start
_start:

  jmp go

msg:
.string "Hello from the grid.\0"

go:
  mov $0, %bx

char:
  mov $0x7c02, %cx

  #movl  $32, %eax
  #movl  (%eax), %eax

  #mov  $32, %ax
  mov  0x7c02(%bx), %ax

  mov $0x0e, %ah
  #mov $'x', %al

  int $0x10

  inc %bl
  mov $0, %al
  cmp %bl, %al
  jne char

hang:
  jmp hang

