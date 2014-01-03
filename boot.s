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

  # All these segments are kernel-level for now. Userspace where?

# This is the first procedure called after completing the switch to protected mode.
# (Might want to re-enable interrupts again?)
in_prot32:
.code32
  # Just print a fancy message here
  mov $(startup_message - _start + 0x7c00), %eax
  call print_str_eax

  # It's unclear what the processor will do when we just stop doing anything here.
  # It'll probably start executing null byte instructions or something else silly.
  # So, just busy loop here.
hang:
  jmp hang

startup_message:
.string "Hello from the grid, speaking to you from protected mode.\0"

# Print the null-terminated string starting at %eax on the first line of the screen.
print_str_eax:
  # Apparently, once we're out of real mode, we can use all the registers however we want.
  mov $0, %ebx
char:
  movb (%eax, %ebx), %ecx
  test %ecx, %ecx
  jz print_done

  # Thank god we don't have to actually mess with pixels.
  # The system provides a video buffer that starts at 0xB8000. For each character (40x25?),
  # 2 bytes are stored; the first byte is the ASCII code, and the second byte is formatting.
  # I read somewhere that the formatting is 4 bits each for foreground and background color,
  # and a value of 7 is grey on black.
  movb %ecx, 0xB8000(,%ebx,2)
  movb $7, 0xB8001(,%ebx,2)

  inc %ebx
  jmp char

print_done:
  ret
