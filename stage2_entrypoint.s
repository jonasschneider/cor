.code64

.section entrypoint
.globl  stage2_entrypoint
stage2_entrypoint:
  mov $kernel_main, %rax
  jmp *%rax
