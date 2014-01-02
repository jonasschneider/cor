.PHONY: all run
all: mbr.bin
CFLAGS= -nostdlib -static -nostartfiles -nodefaultlibs -m32

run: all
	qemu-system-x86_64 -s mbr.bin

boot.o: boot.s
	gcc $(CFLAGS) boot.s -o boot.o

boot.bin: boot.o
	objcopy --only-section=.text -O binary boot.o boot.bin

mbr.bin: boot.bin blank_mbr
	cp blank_mbr mbr.bin
	dd if=boot.bin of=mbr.bin conv=notrunc
