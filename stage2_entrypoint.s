.section entrypoint
.globl  stage2_entrypoint
stage2_entrypoint:
  # MOV isn't enough, you need to actually use MOVABS to load a 64-bit register.
  movabs $kernel_main, %rax
  jmp *%rax
