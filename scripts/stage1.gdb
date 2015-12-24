set confirm off
set pagination off
set disassemble-next-line on
set arch i386
tar rem :1234
break *0x100000
b *0x1000a1
c
c
