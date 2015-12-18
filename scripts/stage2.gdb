set disassemble-next-line on
set arch i386:x86-64
symbol-file stage2.o
tar rem :1234
set variable resume_boot_marker = 1
