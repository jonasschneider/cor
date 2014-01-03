.PHONY: all run
all: mbr.bin stage2.bin
CFLAGS=-nostdlib -static -nostartfiles -nodefaultlibs -Wall -Wextra -m64
CFLAGS32=-nostdlib -static -nostartfiles -nodefaultlibs -Wall -Wextra
CC=./sshwrap gcc
OBJCOPY=./sshwrap objcopy
LD=./sshwrap ld

run: all
	qemu-system-x86_64 -s mbr.bin

boot.o: boot.s
	$(CC) $(CFLAGS32) boot.s -o boot.o

mbr.o: boot.o boot.ldscript
	$(LD) boot.o -T boot.ldscript -o mbr.o

mbr.bin: mbr.o blank_mbr
	cp blank_mbr mbr.bin
	$(OBJCOPY) --only-section=.text -O binary boot.o mbr_contents.bin
	dd if=mbr_contents.bin of=mbr.bin conv=notrunc
	rm mbr_contents.bin

stage2_entrypoint.o: stage2_entrypoint.s
	$(CC) $(CFLAGS) -c stage2_entrypoint.s -o stage2_entrypoint.o

stage2.bin: main.o linkerscript stage2_entrypoint.o
	$(LD) main.o stage2_entrypoint.o -T linkerscript -o stage2.bin
