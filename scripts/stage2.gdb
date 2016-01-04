set disassemble-next-line on
symbol-file target/x86_64-none-elf/release/cor.elf
set arch i386:x86-64
tar rem 172.28.128.1:1234
set variable resume_boot_marker = 1
