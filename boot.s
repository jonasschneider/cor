# Tell the assembler to output 16-bit code; x86-compatible CPUs start in the 16-bit Real Mode.
.code16gcc

# Export the _start symbol, which is by convention the entry point for the .text
# section. Our makefile places the beginning of the text section at the start of
# the MBR. At runtime, the BIOS loads the MBR into address 0x7c00.
.globl  _start
_start:
  # Apparently, the 20th bit of memory addresses is always disabled on PCs by default
  # because of ridiculous backwards compat. Also, enabling/disabling this
  # behaviour is managed by the keyboard controller (!!!).
  # We'll pretend that we didn't hear this, enable the A20 line and move on.
  in $0x92, %al
  or $2, %al
  out %al, $0x92

  # Let's enter protected mode.

  # Disable interrupts - apparently that's a thing.
  cli

  # So protected mode has this thing called segmentation. A table of segments
  # contains a list of segment descriptors. These are blobs of virtual memory
  # of arbitrary size that are mapped to physical memory by the MMU.
  # Additionally, segmentation can provide write and execute protection for
  # some segments.
  # It looks like Real OS's (tm) only touch segmentation as little as possible,
  # and generally try to use paging for protection.

  # Load our Global Descriptor Table (GDT). This table contains our segment descriptors.
  # We only use segementation very lightly. The addressing calculation is a bit of linker
  # cheating - we're basically telling the assemlber that we know that _start is going
  # to be at 0x7c00, and that we want all addresses calculated relative to that known
  # point of information.
  lgdt (gdt_descriptor - _start + 0x7c00)

  # To formally enter protected mode, we set the protected bit on the CR0.
  # http://en.wikipedia.org/wiki/Control_register
  mov %cr0, %eax
  or $1, %eax # set Protected bit #0
  mov %eax, %cr0

  # Now we are half-way in protected mode - our current segment (CS register) is still 0
  # though, which is murkily defined to be somewhat illegal.
  # Before we fix that, though, we have to clear the CPU's instruction pipeline, since
  # otherwise half-decoded instructions after here will still be executed in Real mode
  # (a microcode thing, apparently), which isn't really all that cool.
  # What better way to do that than a no-op jump!
  jmp after_clear_pipeline
  after_clear_pipeline:

  # Now we finally get ready to make the jump to our first real segment.
  # To do this, we do a long jump. Long jumping means that we're breaking the abstraction of
  # virtual memory by switching segments.
  # The arguments to ljmp are the segment selector, and the offset.
  # Concatenated, they probably form a virtual memory address.
  # Since all our segments map to the entire physical memory, we want to
  # The segment selector is 13 bits of an index into the GDT, 1 bit that is set
  # if we're looking at the LDT, and 2 bits that specify the protection level to access.
  # Since we don't have any Local Descriptor Tables (LDTs) set, we want to look at the GDT,
  # and we want to stay in Protection Level 0, the innermost ring.
  # Our index into the GDT is 1 (the kernel code segment that comes past the null segment).
  # This will also cause the CPU to switch out of 16-bit mode; the instruction that we'll
  # arrive at after the jump will be decoded as 32-bit.
  # We'd have to do the inverse of the MMU's work here, but since within a segment,
  # physical and logical addresses are identical, we can just pass the location of our jump target.
  # So long, 8086 mode!
  ljmp $0b1000, $(in_prot32 - _start + 0x7c00)

# This is some strange metadata struct that points to the GDT.
gdt_descriptor:
  .word (3 * 8) - 1 # GDT size in bytes - 1, 3 is the number of entries
  .word gdt - _start + 0x7c00

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

gdt64_descriptor:
  .word (3 * 8) - 1 # GDT size in bytes - 1, 3 is the number of entries
  .word gdt64 - _start + 0x7c00

gdt64:
  # Null Descriptor that just does nothing
  .word 0x0000, 0x0000
  .byte 0x00, 0b00000000, 0b00000000, 0x00
  # Code Descriptor, this segment is executable, but read-only (Long mode bit set)
  .word 0xffff, 0x0000
  .byte 0x00, 0b10011010, 0b11101111, 0x00
  # Data Descriptor, this segment is writable, but not executable
  .word 0xffff, 0x0000
  .byte 0x00, 0b10010010, 0b11001111, 0x00


# This is the first procedure called after completing the switch to protected mode.
# (Might want to re-enable interrupts again?)
in_prot32:
.code32
  # Actually, we don't do anything fancy in protected mode besides preparing the switch
  # to long mode. Maybe we could skip protected mode altogether, but meh.

  # We're going to set up our page tables starting at 0x1000.
  # CR3 holds the address to the topmost page directory (there are 4 levels)
  # Nb: it's OK to set this here already (even though nothing's really set up yet)
  # because paging is still disabled.

  # Ooh. I wanted to just do this:
  #   movl $0x1000, %cr3
  # but apparently you can't load an immediate value (=a constant) into these control
  # registers, you have to load them in one of the other registers first. Ok.

  movl $0x1000, %edi
  mov %edi, %cr3

  # Zero out 0x1000-0x4FFF.
  # The rep thing apparently writes %eax to %ecx units of memory, starting at %edi.
  # It also increments %edi for each byte written.
  movl $0x1000, %edi
  movl $0, %eax
  movl $(0x4000 / 4), %ecx
  rep stosl

  # Reset %edi back to the beginning of the highest-level page table.
  movl $0x1000, %edi

  # Now write out one page table entry on each level, linking to the next table.
  # Add three to the destination address to set the two lower bits,
  # which cause the page to be Present, Readable, and Writable. (?)
  movl $0x2003, (%edi)
  add $0x1000, %edi
  movl $0x3003, (%edi)
  add $0x1000, %edi
  movl $0x4003, (%edi)
  add $0x1000, %edi

  # Now all that's left is the innermost level.
  # We want to identity map the first megabyte. %edi points to the
  # address of the first page table entry.
  # We won't care about any other pages for now.

  # This is the page table entry for the first page, it has offset 0 and
  # the same bits as above set.
  mov $0x00000003, %ebx

  # Run the block 512 times.
  mov $512, %ecx

next_page_entry:
  # write out the next page descriptor
  movl %ebx, (%edi)

  # move our target to the next page
  add $0x1000, %ebx
  # and move to the next page table entry, each entry is 8 bytes
  add $8, %edi

  # Exit when ecx=0.
  # Loop(t) = if(--%ecx > 0) { goto t; }
  loop next_page_entry

  # Next, enable PAE (physical address extension).
  # This is done by setting bit 5 of CR4.
  # Why do we need this? I don't know.
  mov %cr4, %eax
  or $(1<<5), %eax
  mov %eax, %cr4

  # To switch to long mode, set the LM bit 8 on the model-specific register "EFER"
  # (extended features something something). The code for EFER is 0xC0000080,
  # and %ecx tells rdmsr and wrmsr which MSR to look at.
  mov $0xC0000080, %ecx
  rdmsr
  or $(1<<8), %eax
  wrmsr

  # Now we are in Long mode, but in the Compatibility mode and not the one
  # that we actually want, the 64 bit mode.

  # Enable paging by setting the PG bit 31 in CR0.
  # Note to self, all of this would break if we weren't identity-mapping the first few pages,
  # since we wouldn't be able to actually load the instructions coming after here.
  mov %cr0, %eax
  or $(1<<31), %eax
  mov %eax, %cr0

  # That was hard! Now we should be able to jump to a 64-bit instruction.
  # (Since this is still all stored in the first meg of memory, we don't have to worry
  # about things breaking right away.)

  # Jump to a 64-bit instruction to switch to 64-bit mode
  lgdt (gdt64_descriptor - _start + 0x7c00)
  ljmp $0b1000, $(in_long64 - _start + 0x7c00)

in_long64:
.code64

  # For some reason, I can't do an absolute jump with an immediate operand.
  mov $0x10000, %rax
  jmp *%rax

broken:
  mov $(startup_message_broken - _start + 0x7c00), %eax
  call print_str_eax

  # It's unclear what the processor will do when we just stop doing anything here.
  # It'll probably start executing null byte instructions or something else silly.
  # So, just busy loop here.
hang:
  jmp hang


startup_message_broken:
.string "Failed to enter 64-bit mode.\0"

# Print the null-terminated string starting at %eax on the first line of the screen.
print_str_eax:
  # Apparently, once we're out of real mode, we can use all the registers however we want.
  mov $0, %ebx
char:
  movb (%eax, %ebx), %cl
  test %cl, %cl
  jz print_done

  # Thank god we don't have to actually mess with pixels.
  # The system provides a video buffer that starts at 0xB8000. For each character (40x25?),
  # 2 bytes are stored; the first byte is the ASCII code, and the second byte is formatting.
  # I read somewhere that the formatting is 4 bits each for foreground and background color,
  # and a value of 7 is grey on black.
  movb %cl, 0xB8000(,%ebx,2)
  movb $7, 0xB8001(,%ebx,2)

  inc %ebx
  jmp char

print_done:
  ret
