# Tell the assembler to output 16-bit code; x86-compatible CPUs start in the 16-bit Real Mode.
.code16gcc

# Export the _start symbol, which is by convention the entry point for the .text
# section. Our makefile places the beginning of the text section at the start of
# the MBR.
.globl  _start
_start:
  # Immediately jump to after the string that follows (it'd be an illegal instruction).
  # The string is placed near the beginning so that it's at a known location
  # (The BIOS loads the MBR into address 0x7c00, and the jump is 16 bits, so the string
  #  is known to start at 0x7c02. Ultra hacky, but ok.)
  jmp go

.string "Hello from the grid.\r\nEntering protected mode...\0"

go:
  mov $0x7c02, %bx
  call print_str_BX

  # Let's enter protected mode.
  # http://wiki.osdev.org/Protected_Mode
  #cli
  #lgdt %gdtr


  # It's unclear what the processor will do when we just stop doing anything here.
  # It'll probably start executing null byte instructions or something else silly.
  # So, just busy loop here.
hang:
  jmp hang



print_str_BX:
  # BX (Base Index) is one of the few 16-bit registers that can be
  # used for address arithmetic. I'm not yet sure why, but it apparently has something to
  # do with segments (x86 in Real Mode has a 20-bit address space, but only 16-bit registers).
  # Cf. http://f.osdev.org/viewtopic.php?f=13&t=18374

  # Load the next byte to print into AL, exit if it's \0.
  mov (%bx), %al
  test %al, %al
  jz print_done

  # Invoke INT10/AH=0E, which is by convention a BIOS call to print the character in AL.
  mov $0x0e, %ah
  int $0x10

  # Advance the counter, and rinse
  inc %bx
  jmp print_str_BX

print_done:
  ret
