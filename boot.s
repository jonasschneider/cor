# Welcome to the Matrix.
# This is where it begins. It's.. the bootloader! Our first instructions ever
# will be executed from here. Our main job is to load the next stage, stage2,
# from the hard disk. It's stored there right behind us on sector 2, the sector
# after the MBR, which where we will be loaded from. Also, we don't want
# anyone else to have to deal with 16-bit or even 32-bit legacy quirks and
# weirdnesses, or at least reduce their influence as much as possible. To do that,
# we'll enter the 64-bit Long Mode and fight so that others don't have to.
# Let's get to it!

# First, tell the assembler to output 16-bit code; x86 CPUs, even the 64-bit ones
# we're targeting, start in the 16-bit Real Mode.
.code16

# Export the _start symbol. By convention, it's the entry point for the .text
# section. Our build process places the beginning of the text section (and
# therefore the stuff that we write here) at the start of the MBR. At runtime,
# the BIOS loads the MBR into address 0x7c00.
# You'll occasionally see 0x7c00's, and strange address calculations like
#
#     (dap - _start + 0x7c00)
#
# floating around this file. This is because I'm not smart enough to figure
# out how to get the compiler to figure this out for me -- we're basically
# telling the assembler that we know that _start is going to be at 0x7c00, and
# that we want all addresses calculated relative to that known point of
# information. So let's try and outsmart the computer. (Let's just hope this
# doesn't become a pattern.)
.globl  _start
_start:
  # First things first, a history lesson you're not going to believe.
  # Apparently, the 20th CPU pin for communicating with the RAM chip on the
  # mainboard is disabled on PCs by default because of ridiculous backwards
  # compat. You need to throw around a switch in order to be able to use that
  # pin for addressing memory. Worse, toggling this switch is managed by the
  # keyboard controller (!!!). We'll pretend that we didn't hear this, enable
  # the A20 line and move on.
  in $0x92, %al
  or $2, %al
  out %al, $0x92

  # Our overall task here is to load the second boot stage from the hard disk
  # into RAM, in accordance with our memory map (see README.md). Thankfully,
  # we have access to a couple of helpful BIOS routines that we can use to
  # make our life easier -- otherwise we'd have to write an IDE driver here.
  # However, the only time* we can use these routines is right now, in real
  # mode, which we'll want to switch out of as soon as possible. You can call
  # these BIOS routines by setting some specific register values, and then
  # invoking a software interrupt using the INT instruction. Here, we'll use
  # the BIOS call "INT 13h/AH=42h: Extended Read Sectors From Drive", which
  # takes a pointer to a "data address packet" or something (DAP) that tells
  # the BIOS which HD blocks to load, and where to put them. The BIOS will
  # then load the sectors from the disk and put them into RAM.
  #
  # (*) There is actually a way to use them after switching away from real
  # mode; it unsurprisingly involves entering real mode again. This means
  # undoing everything we're doing here. I really hope to avoid that.
  # Apparently, another way to do it is to copy the BIOS code somewhere, and
  # later run an emulator to figure out how the BIOS would do it, and then
  # reverse-engineer that at runtime. No comment.

  # TODO: a few words about real mode segmentation

  # A little aside on 8- and 16-bit registers: AL, AH and AX are not separate
  # registers. AX is just composed of the two. So if you load AH with
  # something, then later load AX with 0, AH will also be 0. By the way,
  # simply ensure the data segment is 0 so that the BIOS loads the DAP from
  # the right position.
  mov $0, %ax
  mov %ax, %ds

  mov $0x42, %ah
  mov $0x80, %dl # Set the drive index (0x80 is first drive)

  # Set the address to our DAP (Ctrl-F for "dap:" to see it)
  movw $(dap - _start + 0x7c00), %si

  # Make the call.
  int $0x13

  # Now the stage2 code & data should be placed at 0x10000 in physical memory.
  # We'll get to that again later.

  # Now, we'll construct a memory map at 0x8000. Usually, the memory address
  # space of a PC has various "holes" somewhere, there might be memory-mapped
  # I/O going on at some magic addresses, et cetera. You don't usually notice
  # this from userland because your address space is virtualized and you only
  # see the "good parts". The good news is that the BIOS knows which parts are
  # usable.
  #
  # We won't actually make use of this information here (we don't need it,
  # anyway, 0x100000 is plenty of bytes!) but we have to construct it here.
  # Again, because BIOS call in Real Mode. Future generations (stages) will
  # thank us.
  #
  # Ref: http://wiki.osdev.org/Detecting_Memory_(x86)#BIOS_Function:_INT_0x15.2C_EAX_.3D_0xE820
  # (a couple of snippets below)

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
  jne broken

  # For the subsequent calls to the function: increment DI by your list entry
  # size, reset EAX to 0xE820, and ECX to 24. When you reach the end of the
  # list, EBX may reset to 0. If you call the function again with EBX = 0, the
  # list will start over. If EBX does not reset to 0, the function will return
  # with Carry set when you try to access the entry after the last valid entry.
next_memmap_entry:
  add $32, %di # even though each entry has 24 bytes, align it to 32
  mov $0xe820, %eax
  mov $24, %ecx
  int $0x15
  jc done
  cmp $0, %ebx
  je done

  jmp next_memmap_entry

done:
  movl $0xDEADBEEF, 32(%di) # place a marker so the reader knows we're done. yes, hax.

  # Cool, that worked. Now we can work on getting the CPU out of the 80s (I
  # believe the "Real" of "Real Mode" stands for "Are you for Real?") The
  # first big step is to enter protected mode. It's "protected" because the
  # hardware knows about a distinction between user space and kernel space,
  # which have different privileges. The sad part is that the more you
  # protect yourself, the harder you have to fight to actually get anything
  # done. But hey.

  # First things first: disable hardware interrupts -- apparently that's a thing.
  # (We won't ever reenable them ourselves.)
  cli

  ## asdf

  # channel 0, lobyte/hibyte, rate generator
  mov 0b00110100, %al                  ;
  out %al, $0x43

  # set reload value
  mov $0x8080, %ax
  out %al, $0x40 # set low byte
  rol $8, %ax # swap low/high bytes (you can't output %ah, apparently)
  out %al, $0x40 # set high byte


  ## disable the APIC, enable the PIC
  mov $0x1B, %ecx
  mov $0, %eax
  wrmsr

  sti


  # So protected mode has this thing called segmentation. A table of segments
  # contains a list of segment descriptors. These are blobs of virtual memory
  # of arbitrary size that are mapped to physical memory by the MMU.
  # Additionally, segmentation can provide write and execute protection for
  # some segments. It looks like modern OS's only touch segmentation as
  # little as possible, and generally try to use paging for protection.
  # However, you can't get around it entirely.

  # Load our Global Descriptor Table (GDT). This table contains the segment
  # descriptors.
  lgdt (gdt_descriptor - _start + 0x7c00)

  # To formally enter protected mode, we set the protected bit on the CR0.
  # http://en.wikipedia.org/wiki/Control_register
  mov %cr0, %eax
  or $1, %eax # set Protected bit #0
  mov %eax, %cr0

  # Now we switch over our program counter to one that actually sits within a
  # segment. How, you may ask? Using a long jump. Long jumping means that
  # we're breaking the abstraction of virtual memory by switching segments.
  # The arguments to ljmp are the segment selector, and the offset.
  # Concatenated, they seem to form a virtual memory address. Since all our
  # segments map to the entire physical memory, the actual offset address
  # doesn't change from non-segmented mode. The segment selector is 13 bits of
  # an index into the GDT, 1 bit that is set if we're looking at the LDT, and
  # 2 bits that specify the protection level to access. Since we don't have
  # any Local Descriptor Tables (LDTs) set, we want to look at the GDT, and we
  # want to stay in Protection Level 0, the innermost ring. (This also means
  # that the segment with the lower 3 bits set to 0 is the offset [in bytes]
  # into the GDT.) Our index into the GDT is 1 (the kernel code segment that
  # comes past the null segment). This will also cause the CPU to switch out
  # of 16-bit mode; the instruction that we'll arrive at after the jump will
  # be decoded as 32-bit. We'd have to do the inverse of the MMU's work here,
  # but since within a segment, physical and logical addresses are identical,
  # we can just pass the location of our jump target. So long, Real Mode!
  ljmp $0b1000, $(in_prot32 - _start + 0x7c00)

  # What follows now is some data structures that are used by the code above
  # and below them. Think of it as an interlude between the acts of this
  # glamorous play. The action resumes at in_prot32.
  # TODO: Maybe move all the data to the end
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
  # We could likely just get rid of the first descriptor entirely, since it's a
  # little-endian subset of the second one. We'll leave it here for now for clarity.
gdt64_descriptor:
  .word (7 * 8) - 1 # GDT size in bytes - 1, 6 is the number of entries
  .int gdt64 - _start + 0x7c00

gdt64_highhalfdescriptor:
  .word (7 * 8) - 1 # GDT size in bytes - 1, 6 is the number of entries
  .quad (gdt64 - _start + 0x7c00)|0x0000008000000000

in_prot32:
  # Okay, we're back! That went over real quick. A nice side effect of the
  # switch to protected mode is that we now can use all the goodness of 32-bit
  # instructions, registers and addresses. Let's tell the assembler about his
  # new toys:
.code32

  # Now we are officially in 32-bit Protected Mode. Our various segment
  # selector registers are still 0, which is murkily defined to be somewhat
  # illegal. Since we're not doing much here besides getting the hell out,
  # this might not be problematic. Pretty much the only thing we're doing is
  # setting up the page tables (we're not even enabling paging here.)

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
  lgdt (gdt64_descriptor - _start + 0x7c00)
  # TODO: Investigate: what happens after you LGDT away the segment you are in
  # right now?

will_enter_longmode64:
  # Okay, that was hard. But now we can jump to our first 64-bit code! Again,
  # we do a long jump since this is a segment switch, and the given segment
  # index points to the 64-bit kernel code segment specified in the gdt64.
  ljmp $0b1000, $(in_long64 - _start + 0x7c00)

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
  lgdt (gdt64_highhalfdescriptor - _start + 0x7c00)

  # Okay, all our mode switching is done. The only thing left to do is run the
  # stage2 kernel. Where possible, we'll refer to stuff using the higher-half
  # memory addresses. We can calculate those here by OR'ing the 'reasonable'
  # virtual (which is the same as physical because we identity-mapped) address
  # with the bulky constant 0x0000008000000000.

  # Check if we successfully loaded the next stage by checking the magic bytes.
  # (They should be 0x1337 in good-aka-Big-endian).
  xor %ax, %ax
  movw 0x6fffe|0x0000008000000000, %ax
  cmp $0x3713, %ax
  jne broken

  # Set up the 64-bit stack to start at 0x9fff0 and grow downwards.
  # Our memory map (see README.md) says that the stack starts at 0x9ffff, but we
  # align to the lower 16-byte boundary. I guess that makes sense.
  mov $0x7fff0|0x0000008000000000, %rax
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
  mov $0x10000|0x0000008000000000, %rax
  jmp *%rax

broken:
  # It's rather unclear how to actually stop the processor in case of failure.
  # Due to space constraints (this whole file, assembled, has to fit in the
  # 510 usable MBR bytes), we don't add any display routines here to signal an
  # error condition, but instead just halt the CPU. RIP.
  hlt
