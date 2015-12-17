.code64

.globl trampoline_to_user_rip
trampoline_to_user_rip:
  .quad 0
.globl trampoline_to_user_rsp
trampoline_to_user_rsp:
  .quad 0
.globl trampoline_to_user_codeseg
trampoline_to_user_codeseg:
  .quad 0

.globl trampoline_to_user
trampoline_to_user:
    # TODO: what about interrupts here?
    movabs trampoline_to_user_codeseg, %rax
    movq %rax, %rcx
    movabs trampoline_to_user_rsp, %rax
    movq %rax, %rbx
    movabs trampoline_to_user_rip, %rax


    # Set up the stack correctly for iretq
    pushq $35 # new stack segment, likely irrelevant
    pushq %rbx
    pushf
    pushq %rcx
    pushq %rax
    iretq

    # TODO: set up ISR so we trampoline back to the following label
trampoline_from_user:
    ret # return back to the kernel task that called trampoline_to_user

