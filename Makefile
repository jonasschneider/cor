.PHONY: all run
all: disk.bin
CFLAGS=-nostdlib -static -nostartfiles -nodefaultlibs -Wall -Wextra -m64
CFLAGS32=-nostdlib -static -nostartfiles -nodefaultlibs -Wall -Wextra
CC=./sshwrap gcc
OBJCOPY=./sshwrap objcopy
LD=./sshwrap ld

run: all
	qemu-system-x86_64 -s disk.bin

boot.o: boot.s
	$(CC) $(CFLAGS32) boot.s -o boot.o

mbr.o: boot.o boot.ldscript
	$(LD) boot.o -T boot.ldscript -o mbr.o

mbr.bin: mbr.o blank_mbr boot.o
	cp blank_mbr mbr.bin
	$(OBJCOPY) --only-section=.text -O binary boot.o mbr_contents.bin
	dd if=mbr_contents.bin of=mbr.bin conv=notrunc
	rm mbr_contents.bin

stage2_entrypoint.o: stage2_entrypoint.s
	$(CC) $(CFLAGS) -c stage2_entrypoint.s -o stage2_entrypoint.o

stage2.o: main.o linkerscript stage2_entrypoint.o
	$(LD) main.o stage2_entrypoint.o -T linkerscript -o stage2.o

stage2.bin: stage2.o
	$(OBJCOPY) --only-section=.text -O binary stage2.o stage2.bin

disk.bin: mbr.bin stage2.o Makefile
	# we want a 0x70000 byte disk
	dd if=/dev/zero of=disk.bin~ conv=notrunc bs=1 count=$$((0x70000))
	$(OBJCOPY) -I elf64-x86-64 --only-section=.text -O binary stage2.o stage2_text~
	$(OBJCOPY) -I elf64-x86-64 --only-section=.data -O binary stage2.o stage2_data~
	# add the MBR.
	dd if=mbr.bin of=disk.bin~ conv=notrunc
	# add the text section
	dd if=stage2_text~ of=disk.bin~ conv=notrunc bs=1 seek=512
	# and add the data. ultra hacky to mess with the offsets here, and it's probably wrong, but meh...
	dd if=stage2_data~ of=disk.bin~ conv=notrunc bs=1 seek=$$((512 + 0x40000))
	mv disk.bin~ disk.bin
