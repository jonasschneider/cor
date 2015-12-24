# Declare constants used for creating a multiboot header.
.set ALIGN,    1<<0             # align loaded modules on page boundaries
.set MEMINFO,  1<<1             # provide memory map
.set FLAGS,    ALIGN|MEMINFO  # no flags instead of ALIGN | MEMINFO
.set MAGIC,    0x1BADB002       # 'magic number' lets bootloader find the header
.set CHECKSUM, -(MAGIC + FLAGS) # checksum of above, to prove we are multiboot

.section .bootstrap_stack, "aw", @nobits
stack_bottom:
.skip 16384 # 16 KiB
stack_top:

.section .bootstart
.global _start
.code32
.type _start, @function
_start:
  jmp go

  .align 4
  .long MAGIC
  .long FLAGS
  .long CHECKSUM

go:
  cli

  lgdt gdt_descriptor

  # So, we're going to set up our page tables starting at 0x1000. The control
  # register CR3 will hold the *physical* address to the topmost page
  # directory (there are 4 levels). We can load this here already, even though
  # we haven't set up the tables yet, because paging is still disabled. We'll
  # get to that later. (CR3 holds the physical address because recursion)

  # Ooh. I wanted to just do this:
  #   movl $0x1000, %cr3
  # but apparently you can't load an immediate value (=a constant) into these control
  # registers, you have to load them in one of the other registers first. Ok.
  movl $0x1000, %edi
  mov %edi, %cr3

  # Zero out 0x1000-0x5FFF. The rep thing apparently writes %eax to %ecx units
  # of memory, starting at %edi. It also increments %edi for each byte
  # written.
  movl $0x1000, %edi
  movl $0, %eax
  movl $(0x5000 / 4), %ecx
  rep stosl

  # Reset %edi back to the beginning of the highest-level page table.
  movl $0x1000, %edi

  # Now we write out a single page table for every level, each linking to the
  # next. Add three to the destination addresses to set the two lower bits,
  # which cause the page to be Present, Readable, and Writable. (?)
  movl $0x2003, (%edi)

  # We are going to construct a simple tree-like page structure here; For now,
  # this just means that we can access physical memory starting at 0 from both
  # virtual 0x0 and virtual 0x8000000000. Later, we'll remove the first
  # mapping and replace it with the address space for our user-space processes
  # (see task.c). When doing this, task.c will copy the top level of the
  # paging structure that we define here, and only mess with the entry we
  # defined above, while leaving other entries, like this one, untouched.
  movl $0x2003, 8(%edi)

  # Add 0x1000 to move %edi to the next-deeper level. For the next two levels,
  # We won't do anything fancy and just link to the next table while setting
  # the same bits as above.
  add $0x1000, %edi
  movl $0x3003, (%edi)
  add $0x1000, %edi
  movl $0x4003, (%edi)

  # Now all that's left is the innermost level. We want to identity map the
  # first megabyte. Set %edi to the address of the first page table entry.
  add $0x1000, %edi

  # This is the page table entry for the first page, it has offset 0 and
  # the same bits as above set.
  mov $0x00000003, %ebx

  # Run the following block 512 times:
  mov $512, %ecx

next_page_entry:
  # write out the next page table entry
  movl %ebx, (%edi)

  # add the offset for the next entry to the output register
  add $0x1000, %ebx
  # and move out "write head" to the next page table entry (each entry is 8 bytes long)
  add $8, %edi

  # Exit when ecx=0; we'll then have written all 512 entries.
  # Loop(t) = if(--%ecx > 0) { goto t; }
  loop next_page_entry

  # Okay, that's it for the page tables!

  # Next, enable PAE (physical address extension). This is done by setting bit
  # 5 of CR4. This allows as to address >4GB of memory, but most importantly
  # it's required for getting into Long Mode.
  mov %cr4, %eax
  or $(1<<5), %eax
  mov %eax, %cr4

  # To enable Long Mode, set the LM bit 8 on the model-specific register "EFER"
  # (extended features something something). The code for EFER is 0xC0000080,
  # and %ecx tells rdmsr and wrmsr which MSR to look at.
  mov $0xC0000080, %ecx
  rdmsr
  or $(1<<8), %eax
  wrmsr

  # Now Long Mode is _enabled_. However, it isn't _active_ yet; right now, we are
  # still in some sort of compatibility mode. We can still start using the 64-bit
  # mode page tables, however, which is what we'll do.

  # Enable paging by setting the PG bit 31 in CR0. Note to self, all of this
  # would break if we weren't identity-mapping the first few pages, since we
  # wouldn't be able to actually load the instructions coming after here.
  # We'd be pulling the rug under our feet away.
  mov %cr0, %eax
  or $(1<<31), %eax
  mov %eax, %cr0

  # Okay. Now we have everything we need to _active_ Long Mode. Whether or not
  # Long Mode is activated is stored by segment (x86_64 doesn't use much of
  # the old x86's segmentation, but it does use it for doing this switch and
  # for protection level management) First, we'll load our new descriptor
  # table containing, among other things, a 64-bit segment for us to jump to.
  lgdt gdt64_descriptor
  # TODO: Investigate: what happens after you LGDT away the segment you are in
  # right now?

will_enter_longmode64:
  # Okay, that was hard. But now we can jump to our first 64-bit code! Again,
  # we do a long jump since this is a segment switch, and the given segment
  # index points to the 64-bit kernel code segment specified in the gdt64.
  ljmp $0b1000, $in_long64

.align 8
in_long64:
.code64
  # Okay, this might seem kind of redundant. We are going to load the same GDT
  # that is already active again. The reason for doing that is during the
  # previous load, we were in 32-bit mode. This means that GDTR, the register
  # containing the address of the GDT, now has a value like 0x7cab (it's
  # actually a bit different since it also stores the size of the GDT.)
  # This will become problematic later when we want to unmap the low kernel memory.
  # To fix it, we'll just load the same table again, but this time addressed as
  # something like 0x80000007cab (zeroes not to scale).
  # It's still the same table, just addressed differently.
  lgdt gdt64_highhalfdescriptor

  # Okay, all our mode switching is done. The only thing left to do is run the
  # stage2 kernel. Where possible, we'll refer to stuff using the higher-half
  # memory addresses. We can calculate those here by OR'ing the 'reasonable'
  # virtual (which is the same as physical because we identity-mapped) address
  # with the bulky constant 0x0000008000000000.

  # Set up the 64-bit stack to start at 0x9fff0 and grow downwards.
  # Our memory map (see README.md) says that the stack starts at 0x9ffff, but we
  # align to the lower 16-byte boundary. I guess that makes sense.
  mov $stack_top, %rax
  mov $0x0000008000000000, %rbx
  or %rbx, %rax
  mov %rax, %rsp

  # Another thing you normally would never even think about disabling, so we'll set
  # it here for you: Set bit 9 of CR4, the OSFXSR bit, to enable SSE instructions
  # and access to the XMM registers.
  mov %cr4, %rax
  or $(1<<9), %rax
  mov %rax, %cr4

  # Finally, go on and jump to stage 2! (Way at the beginning, loaded it from
  # disk and placed its entrypoint at 0x10000.) For some reason, I can't do an
  # absolute jump with an immediate operand. Our job here is done, we'll go
  # and never return.
  movabs $kernel_main, %rax
  mov $0x0000008000000000, %rbx
  or %rbx, %rax
  jmp *%rax

broken:
  # It's rather unclear how to actually stop the processor in case of failure.
  # Due to space constraints (this whole file, assembled, has to fit in the
  # 510 usable MBR bytes), we don't add any display routines here to signal an
  # error condition, but instead just halt the CPU. RIP.
  cli
  hlt


# This is some strange metadata struct that points to the GDT.
gdt_descriptor:
  .word (3 * 8) - 1 # GDT size in bytes - 1, 3 is the number of entries
  .long gdt

gdt:
  # The fairly ridiculous GDT format is as follows:
  # Cf. http://www.cs.cmu.edu/~410/doc/segments/segments.html
  # (Don't get confused by endianness, I certainly did.)

  # - 16 low bits of limit
  # - 16 low bits of base
  # - 8 middle bits of base
  # - 8 access bits
  # - 4 middle bits of limit
  # - 4 granularity bits (?)
  # - 8 high bits of base

  # Null Descriptor that just does nothing
  .word 0x0000, 0x0000
  .byte 0x00, 0b00000000, 0b00000000, 0x00
  # Code Descriptor, this segment is executable, but read-only
  .word 0xffff, 0x0000
  .byte 0x00, 0b10011010, 0b11001111, 0x00
  # Data Descriptor, this segment is writable, but not executable
  .word 0xffff, 0x0000
  .byte 0x00, 0b10010010, 0b11001111, 0x00

.align 8 # TODO: unclear whether we actually need these
gdt64:
  # TODO: Maybe we could actually leave out 32-bit GDT entirely;
  # the Long Mode bits should be ignored by them.

  # Null Descriptor that just does nothing
  .word 0x0000, 0x0000
  .byte 0x00, 0b00000000, 0b00000000, 0x00
  # Code Descriptor, this segment is executable, but read-only (Long mode bit set)
  .word 0xffff, 0x0000
  .byte 0x00, 0b10011010, 0b10101111, 0x00
  # Data Descriptor, this segment is writable, but not executable
  .word 0xffff, 0x0000
  .byte 0x00, 0b10010010, 0b11001111, 0x00

  # Code descriptor for userspace
  .word 0xffff, 0x0000
  .byte 0x00, 0b11111110, 0b10101111, 0x00
  # Data Descriptor for userspace
  # TODO: I don't think this actually needed, either QEMU or I get x86-64 wrong
  .word 0xffff, 0x0000
  .byte 0x00, 0b11110010, 0b11001111, 0x00

  # TSS descriptor (there is only one TSS in 64-bit)
  # Tell everyone that the TSS lives at 0x80000|0x0000008000000000 (tss.c cares)
  .word 0x67, 0x0000
  .byte 0x08, 0b11101001, 0b00000000, 0x00
  .quad 0x0000008000000000>>32
  .word 0x00, 0x00

.align 4 # TODO: unclear whether we actually need this
gdt64_descriptor:
  .word (7 * 8) - 1 # GDT size in bytes - 1, 6 is the number of entries
  .long gdt64
  .long 0

.align 4
gdt64_highhalfdescriptor:
  .word (7 * 8) - 1 # GDT size in bytes - 1, 6 is the number of entries
  .long gdt64
  .long 0x80



.size _start, . - _start
