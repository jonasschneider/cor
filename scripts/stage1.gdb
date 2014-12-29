add-symbol-file boot.o 0x7c00
set disassemble-next-line on
set arch i386
tar rem :1234
break *0x7c00
c
