set disassemble-next-line on
symbol-file cor.elf 0x100000
set arch i386:x86-64
tar rem :1234
set variable resume_boot_marker = 1
