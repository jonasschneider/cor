.globl _start
_start:
  mov $0x7fffffffe000, %rsp
  jmp cmod_main
