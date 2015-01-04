.code64

.section entrypoint
.globl  stage2_entrypoint
stage2_entrypoint:
  movabs $kernel_main, %rax
  jmp *%rax
