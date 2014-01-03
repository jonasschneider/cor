.PHONY: all run
all: mbr.bin stage2.bin
CFLAGS=-nostdlib -static -nostartfiles -nodefaultlibs -Wall -Wextra
CFLAGS32=$(CFLAGS)
CC=./sshwrap gcc
OBJCOPY=./sshwrap objcopy
LD=./sshwrap ld

run: all
	qemu-system-x86_64 -s mbr.bin

boot.o: boot.s
	$(CC) $(CFLAGS32) boot.s -o boot.o

boot.bin: boot.o
	$(OBJCOPY) --only-section=.text -O binary boot.o boot.bin

stage2.bin: main.o linkerscript
	$(LD) main.o -T linkerscript -o stage2.bin

mbr.bin: boot.bin blank_mbr
	cp blank_mbr mbr.bin
	dd if=boot.bin of=mbr.bin conv=notrunc
