	.section	__TEXT,__text,regular,pure_instructions
	.globl	_main
	.align	4, 0x90
_main:                                  ## @main
## BB#0:
	pushl	%ebp
	movl	%esp, %ebp
	pushl	%eax
	movl	$0, %eax
	movl	$507906, %ecx           ## imm = 0x7C002
	movl	(%ecx), %ecx
	movl	%ecx, -4(%ebp)
	addl	$4, %esp
	popl	%ebp
	ret


.subsections_via_symbols
