.code64

# Statically pass these arguments in here because I'm stupid
.globl context_switch_oldrsp_dst
context_switch_oldrsp_dst:
  .quad 0
.globl context_switch_newrsp
context_switch_newrsp:
  .quad 0
.globl context_switch_jumpto
context_switch_jumpto:
  .quad 0

.globl context_switch
context_switch:
    # Well, the context switch itself is a bit hairy. Our goals are:
    #
    #  1. Save the old task's %rsp and %rbp into its task struct so we know
    #     how to resume
    #  2. Load the new task's %rsp and %rbp from its task struct
    #  3. And then comes something interesting:
    #    - If the new task has never been launched yet: "Call" its entrypoint
    #      -- I say "call" because since we're not in a valid stack frame,
    #      this call can never return without breaking stuff. So, we wrap it
    #      in a call around starttask() which will rather panic than return.
    #    - Otherwise, the new stack frame puts us into a call to
    #      context_switch itself, the frame that called us is code from within
    #      the resumed task. So, we should just return like a reasonable
    #      citizen.

    # awfully, we can only movabs into %rax.
    # TODO(perf): the entire method of passing parameters here.. can't rust just be smart about it?
    movabs context_switch_newrsp, %rax
    mov %rax, %r8
    movabs context_switch_jumpto, %rax
    mov %rax, %r10
    movabs context_switch_oldrsp_dst, %rax

    cmp $0, %rax
    je context_switch_discard_old # skip saving the old values, this is only needed when a task exits
    push %rbp
    mov %rsp, (%rax)
  context_switch_discard_old:
    mov %r8, %rsp
    cmp $0, %r10
    jne context_switch_jmp_to_starttask

    pop %rbp
    ret # this will "return" to the new stack

  context_switch_jmp_to_starttask:
    mov %rsp, %rbp
    jmp *%r10

