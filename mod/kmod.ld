SECTIONS
{
  . = 0x40000;
  .text : { *(.text*) }
  .data : { *(.data) }
  .bss : { *(.bss) }
}
