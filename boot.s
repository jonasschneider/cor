# Tell the assembler to output 16-bit code; x86-compatible CPUs start in the 16-bit Real Mode.
.code16

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

  # Our task here is to load the second boot stage into RAM, according to our memory map.
  # We could write our own IDE drivers, but we can also just use the BIOS while we're still
  # in protected mode.
  # We'll use "INT 13h/AH=42h: Extended Read Sectors From Drive", which takes a pointer
  # to a "data address packet" or something (DAP), which tells the BIOS which HD blocks to
  # load where. We'll use LBA and not the traditional cylinder, head, sector addressing
  # because who needs backwards compat.

  # Learned here the hard way: AL, AH and AX are not separate registers. AX is just composed
  # of the two. So if you load AH with something, then later load AX with 0, AH will also be 0.
  # By the way, simply ensure the data segment is 0 so that the BIOS loads the DAP from the
  # right position.
  mov $0, %ax
  mov %ax, %ds

  mov $0x42, %ah
  mov $0x80, %dl # Set the drive index (0x80 is first drive)

  # Set the address to our DAP
  movw $(dap - _start + 0x7c00), %si

  int $0x13

  # Try to construct a memory map at 0x8000.
  # http://wiki.osdev.org/Detecting_Memory_(x86)#BIOS_Function:_INT_0x15.2C_EAX_.3D_0xE820

  # For the first call to the function, point ES:DI at the destination buffer
  # for the list. Clear EBX. Set EDX to the magic number 0x534D4150. Set EAX to
  # 0xE820 (note that the upper 16-bits of EAX should be set to 0). Set ECX to
  # 24. Do an INT 0x15.
  mov $0x0000, %ax
  mov %ax, %es
  mov $0x8000, %di
  mov $0x534d4150, %edx
  mov $0, %ebx
  mov $0xe820, %eax
  mov $24, %ecx
  int $0x15

  # If the first call to the function is successful, EAX will be set to
  # 0x534D4150, and the Carry flag will be clear. EBX will be set to some non-zero
  # value, which must be preserved for the next call to the function. CL will
  # contain the number of bytes actually stored at ES:DI (probably 20).
  cmp $0x534d4150, %eax
  jne failed

  # For the subsequent calls to the function: increment DI by your list entry
  # size, reset EAX to 0xE820, and ECX to 24. When you reach the end of the
  # list, EBX may reset to 0. If you call the function again with EBX = 0, the
  # list will start over. If EBX does not reset to 0, the function will return
  # with Carry set when you try to access the entry after the last valid entry.
next:
  add $32, %di # even though each entry has 24 bytes, align it to 32
  mov $0xe820, %eax
  mov $24, %ecx
  int $0x15
  jc done
  cmp $0, %ebx
  je done

  jmp next

failed: jmp failed # TODO: error message etc

done:
  movl $0xDEADBEEF, 32(%di) # place a marker so the reader knows we're done. yes, hax.

  # Okay, we created the memory map for the next stage.
  # (We're not going to make any use of it here.)

  # Now, let's enter protected mode.

  # First things first: disable interrupts -- apparently that's a thing.
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

  # Now we switch over our program counter to one that actually sits within a segment.
  # To do this, we do a long jump. Long jumping means that we're breaking the abstraction of
  # virtual memory by switching segments.
  # The arguments to ljmp are the segment selector, and the offset.
  # Concatenated, they seem to form a virtual memory address.
  # Since all our segments map to the entire physical memory, we want to
  # The segment selector is 13 bits of an index into the GDT, 1 bit that is set
  # if we're looking at the LDT, and 2 bits that specify the protection level to access.
  # Since we don't have any Local Descriptor Tables (LDTs) set, we want to look at the GDT,
  # and we want to stay in Protection Level 0, the innermost ring.
  # (This also means that the segment with the lower 3 bits set to 0 is the offset [in bytes] into the GDT.)
  # Our index into the GDT is 1 (the kernel code segment that comes past the null segment).
  # This will also cause the CPU to switch out of 16-bit mode; the instruction that we'll
  # arrive at after the jump will be decoded as 32-bit.
  # We'd have to do the inverse of the MMU's work here, but since within a segment,
  # physical and logical addresses are identical, we can just pass the location of our jump target.
  # So long, 8086 mode!
  ljmp $0b1000, $(in_prot32 - _start + 0x7c00)

dap:
  .byte 0x10 # struct size (16 bytes)
  .byte 0 # reserved 0
  .word (0x60000 / 512) # number of sectors to read

   # target position for read, offset then segment because of little endian
  .word 0x0000
  .word 0x1000

  .long 1 # first sector to read
  .long 0 # high address

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

.align 8 # TODO: unclear whether we actually need these
gdt64:
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
  # Data Descriptor for userspace, actually I don't think this i needed, but qemu gets x86-64 wrong
  .word 0xffff, 0x0000
  .byte 0x00, 0b11110010, 0b11001111, 0x00

  # TSS descriptor (there is only one TSS in 64-bit)
  # Place the TSS itself at 0x80000|0x0000008000000000 (tss.c cares)
  .word 0x67, 0x0000
  .byte 0x08, 0b11101001, 0b00000000, 0x00
  .quad 0x0000008000000000>>32
  .word 0x00, 0x00

.align 4 # TODO: unclear whether we actually need these
gdt64_descriptor:
  .word (7 * 8) - 1 # GDT size in bytes - 1, 6 is the number of entries
  .int gdt64 - _start + 0x7c00

gdt64_highhalfdescriptor:
  .word (7 * 8) - 1 # GDT size in bytes - 1, 6 is the number of entries
  .quad (gdt64 - _start + 0x7c00)|0x0000008000000000


# This is the first procedure called after completing the switch to protected mode.
# (Might want to re-enable interrupts again?)
in_prot32:
.code32
  # Now we are half-way in protected mode - our various segment selector registers are still 0
  # though, which is murkily defined to be somewhat illegal.

  # However, we'll set up the stack for stage2 once we get into 64 bit mode.

  # Now, we are in protected mode, but we want to enable paging.
  # This is because 64-bit mode supports paging only, and paging > segmentation anyhow.
  # For some reason.
  # So, we're going to set up our page tables starting at 0x1000.
  # CR3 holds the address to the topmost page directory (there are 4 levels)
  # Nb: it's OK to set this here already (even though nothing's really set up yet)
  # because paging is still disabled.

  # Ooh. I wanted to just do this:
  #   movl $0x1000, %cr3
  # but apparently you can't load an immediate value (=a constant) into these control
  # registers, you have to load them in one of the other registers first. Ok.

  movl $0x1000, %edi
  mov %edi, %cr3

  # Zero out 0x1000-0x5FFF.
  # The rep thing apparently writes %eax to %ecx units of memory, starting at %edi.
  # It also increments %edi for each byte written.
  movl $0x1000, %edi
  movl $0, %eax
  movl $(0x5000 / 4), %ecx
  rep stosl

  # Reset %edi back to the beginning of the highest-level page table.
  movl $0x1000, %edi

  # Now write out one page table entry on each level, linking to the next table.
  # Add three to the destination address to set the two lower bits,
  # which cause the page to be Present, Readable, and Writable. (?)
  movl $0x2003|4, (%edi)
  movl $0x2003|4, 8(%edi) # map the kernel here again
  add $0x1000, %edi
  movl $0x3003|4, (%edi)
  add $0x1000, %edi
  movl $0x4003|4, (%edi)
  movl $0x5003|4, 16(%edi)
  add $0x1000, %edi

  # Now all that's left is the innermost level.
  # We want to identity map the first megabyte. %edi points to the
  # address of the first page table entry.
  # We won't care about any other pages for now.

  # This is the page table entry for the first page, it has offset 0 and
  # the same bits as above set.
  mov $0x00000003|4, %ebx

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

  # write page table entry for stage2.init
  movl $0x70003|4, 0x5000


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
will_enter_longmode64:
  ljmp $0b1000, $(in_long64 - _start + 0x7c00)

in_long64:
.code64
  # Now load the second gdt64 descriptor that references the higher-half GDT
  lgdt (gdt64_highhalfdescriptor - _start + 0x7c00)

  # Check if we successfully loaded the next stage by checking the magic bytes
  xor %ax, %ax
  movw 0x6fffe, %ax
  cmp $0x3713, %ax
  jne broken

  # Set up the 64-bit stack to be .. somewhere in higher memory.
  # FIXME!! let's just hope this doesn't break anything for now
  mov $0x70000|0x0000008000000000, %rax
  mov %rax, %rsp

  # Jump to stage 2!
  # For some reason, I can't do an absolute jump with an immediate operand.
  mov $0x10000|0x0000008000000000, %rax
  jmp *%rax

broken:
  # It's unclear what the processor will do when we just stop doing anything here.
  # It'll probably start executing null byte instructions or something else silly.
  # So, just busy loop here.
hang:
  jmp hang
