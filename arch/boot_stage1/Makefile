ROOT=../..
include ../../Makefile.conf
.PHONY: clean

mbr.bin: blank_mbr boot.o
	cp blank_mbr mbr.bin
	$(OBJCOPY) --only-section=.text -O binary boot.o mbr_contents.bin
	dd if=mbr_contents.bin of=mbr.bin conv=notrunc
	rm mbr_contents.bin

boot.o: boot.s
	$(AS) boot.s -o boot.o

clean:
	rm -f boot.o mbr.bin
