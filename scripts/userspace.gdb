set confirm off
set disassemble-next-line on
set arch i386:x86-64
symbol-file stage2.o
add-symbol-file userspace/init 0x0000000000400144
tar rem :1234
b _start
c
