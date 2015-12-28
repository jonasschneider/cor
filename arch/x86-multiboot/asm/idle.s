.code64

.globl asm_idle
asm_idle:
    sti
    hlt
    ret

.globl asm_abort
asm_abort:
  cli
  hlt
