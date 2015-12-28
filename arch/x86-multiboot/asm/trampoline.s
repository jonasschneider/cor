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



trampoline_previous_kernel_rsp:
  .quad 0

.globl trampoline_to_user
trampoline_to_user:
    # TODO: what about interrupts in here?

    push %rbp
    push %rbx
    push %r12
    push %r13
    push %r14
    push %r15
    mov %rsp, %rax
    movabs %rax, trampoline_previous_kernel_rsp

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


.globl trampoline_from_user
trampoline_from_user:
    movabs %rax, trampoline_from_user_arg1
    mov %rbx, %rax
    movabs %rax, trampoline_from_user_arg2
    mov %rcx, %rax
    movabs %rax, trampoline_from_user_arg3
    mov %rdx, %rax
    movabs %rax, trampoline_from_user_arg4

    mov (%rsp), %rax
    movabs %rax, trampoline_from_user_rip
    mov 8(%rsp), %rax
    movabs %rax, trampoline_from_user_codeseg
    mov 24(%rsp), %rax
    movabs %rax, trampoline_from_user_rsp

    movabs trampoline_previous_kernel_rsp, %rax
    mov %rax, %rsp

    pop %r15
    pop %r14
    pop %r13
    pop %r12
    pop %rbx
    pop %rbp

    ret # return back to the kernel task that called trampoline_to_user

.globl trampoline_from_user_arg1
trampoline_from_user_arg1:
  .quad 0
.globl trampoline_from_user_arg2
trampoline_from_user_arg2:
  .quad 0
.globl trampoline_from_user_arg3
trampoline_from_user_arg3:
  .quad 0
.globl trampoline_from_user_arg4
trampoline_from_user_arg4:
  .quad 0


.globl trampoline_from_user_rip
trampoline_from_user_rip:
  .quad 0
.globl trampoline_from_user_rsp
trampoline_from_user_rsp:
  .quad 0
.globl trampoline_from_user_codeseg
trampoline_from_user_codeseg:
  .quad 0
