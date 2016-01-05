ROOT=../..
include ../../Makefile.conf
CC=x86_64-elf-gcc
PREFIX=../../target/x86_64-none-elf/debug
.PHONY: clean

$(PREFIX)/cor-x86_64-multiboot.iso: $(PREFIX)/cor.elf
	mkdir -p $(PREFIX)/iso/boot/grub
	cp grub.cfg $(PREFIX)/iso/boot/grub
	cp $(PREFIX)/cor.elf $(PREFIX)/iso/boot
	grub-mkrescue -d /usr/lib/grub/i386-pc/ -o $@ $(PREFIX)/iso

$(PREFIX)/libcor.a: $(shell find ../../src)
	cargo rustc --target=`pwd`/x86_64-none-elf.json -- -C no-stack-check -C relocation-model=static -C code-model=large -Z no-landing-pads

$(PREFIX)/cor.elf: boot.o asm/* runtime.o $(PREFIX)/libcor.a
	env CC=$(CC) make -C asm
	$(CC) $(CFLAGS) -mcmodel=large -Wl,-n,--build-id=none,--gc-sections -nostdlib -ffreestanding -Os -o $@ asm/*.o runtime.o -L$(PREFIX) -lcor boot.o -T multiboot.ld

boot.o: boot.s
	$(AS) boot.s -o boot.o

runtime.o: ../../src/runtime.c
	$(CC) -o $@ $< -c $(KCCFLAGS)

clean:
	make -C asm clean
	rm -f boot.o runtime.o $(PREFIX)/cor-x86_64-multiboot.iso $(PREFIX)/cor.elf
