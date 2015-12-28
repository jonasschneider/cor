/* The bootloader will look at this image and start execution at the symbol
   designated as the entry point. */
ENTRY(_start)

/* Tell where the various sections of the object files will be put in the final
   kernel image. */
SECTIONS
{
  /* Begin putting sections at 1 MiB, a conventional place for kernels to be
     loaded at by the bootloader. */
  . = 0x100000;

  /* First put the multiboot header, as it is required to be put very early
     early in the image or the bootloader won't recognize the file format.
     Next we'll put the .text section. */
  .multiboot :
  {
    *(.multiboot)
    *(.bootstart)
    *(.bootstrap_stack)
  }

  /* Higher-half stuff goes here */
  . = 0x8000130000;

  .text : AT(0x130000)
  {
    *(.text)
    *(.text*)
  }

  . = 0x80001a0000;

  .data : AT(0x1a0000) {
    *(.rodata*)
    *(.data*)
    *(COMMON)
    *(.bss)
  }

  . = 0x8000200000;

  /DISCARD/ : { *(.eh_frame) }

  /* The compiler may produce other sections, by default it will put them in
     a segment with the same name. Simply add stuff here as needed. */
}
