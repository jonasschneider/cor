ROOT=.
include Makefile.conf
.PHONY: all clean

OBJS=main.o printk.o chrdev_serial.o chrdev_console.o io.o elf.o interrupthandler.o tss.o mm.o task.o pci.o timer.o pic.o interrupt.o test_mock_supplement.o context_switch.o

all: disk.bin

clean:
	rm -f *.o *.bin *~ init *.so
	$(MAKE) -C userspace clean
	$(MAKE) -C src clean

%.o: %.c
	$(CC) $(KCCFLAGS) $< -c -o $@

boot.o: boot.s
	$(AS) boot.s -o boot.o

src/%.kmo: src/**/*.rs src/*.rs src/*.c
	$(MAKE) -C src

mbr.bin: blank_mbr boot.o
	cp blank_mbr mbr.bin
	$(OBJCOPY) --only-section=.text -O binary boot.o mbr_contents.bin
	dd if=mbr_contents.bin of=mbr.bin conv=notrunc
	rm mbr_contents.bin

stage2_entrypoint.o: stage2_entrypoint.s
	$(CC) $(KCFLAGS) -c stage2_entrypoint.s -o stage2_entrypoint.o

context_switch.o: context_switch.s
	$(CC) $(KCFLAGS) -c $< -o $@

interrupthandler.o: interrupthandler.s include/cor/syscall.h intstubs.s~
	$(CC) $(KCFLAGS) -c -x assembler-with-cpp -Iinclude $< -o $@

test_mock_supplement.o: $(wildcard ./test_mock_supplement.c~) test_mock_supplement_stub.c
	if [ -e "test_mock_supplement.c~" ]; then f="test_mock_supplement.c~"; else f="test_mock_supplement_stub.c"; fi; $(CC) $(KCCFLAGS) -c -x c $$f -o $@

stage2.o: $(OBJS) linkerscript stage2_entrypoint.o init_static.o src/block.kmo
	echo LONG\(0x$(shell git rev-parse HEAD | cut -c 1-6)\) > versionstamp~
	# -x here removes local symbols (like those from hello.kmo from the file -- maybe for "production"?)
	$(LD) $(OBJS) stage2_entrypoint.o init_static.o src/block.kmo -T linkerscript -o stage2.o

stage2.bin: stage2.o
	$(OBJCOPY) --only-section=.text -O binary stage2.o stage2.bin

disk.bin: mbr.bin stage2.o Makefile
	# we want a 0x60100 byte disk
	dd if=/dev/zero of=disk.bin~ conv=notrunc bs=512 count=$$((0x60100 / 512))
	$(OBJCOPY) -I elf64-x86-64 --only-section=.text -O binary stage2.o stage2_text~
	$(OBJCOPY) -I elf64-x86-64 --only-section=.data -O binary stage2.o stage2_data~
	# add the MBR.
	dd if=mbr.bin of=disk.bin~ conv=notrunc
	# add the text section
	dd if=stage2_text~ of=disk.bin~ conv=notrunc bs=512 seek=1
	# and add the data. ultra hacky to mess with the offsets here, and it's probably wrong, but meh...
	dd if=stage2_data~ of=disk.bin~ conv=notrunc bs=512 seek=$$((1 + (0x40000/512)))
	mv disk.bin~ disk.bin

init_static.o: init_static.c~
	$(CC) $(KCCFLAGS) -c -x c $< -o $@

init_static.c~: userspace/*.c
	$(MAKE) -C userspace
	cat userspace/init | ruby -e 'b = $$stdin.read.bytes; puts "int cor_stage2_init_data_len = "+b.count.to_s+"; char cor_stage2_init_data[] = {";puts b.map{|x|"0x#{x.to_s(16)}"}.join(", ");puts "};"' > $@

intstubs.s~:
	ruby -e '0.upto(255) { |i| puts ".align 16\n.global intrstub_#{i}\nintrstub_#{i}:\n  push $$#{i}\n  jmp isr_dispatcher\n\n" }' > $@
