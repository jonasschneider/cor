ROOT=.
include Makefile.conf
.PHONY: all clean

OBJS=main.o printk.o chrdev_serial.o chrdev_console.o io.o interrupthandler.o tss.o mm.o task.o pci.o timer.o pic.o interrupt.o test_mock_supplement.o
OBJS+=context_switch.o trampoline.o idle.o

all:
	$(MAKE) -C src/
	$(MAKE) disk.bin
	$(MAKE) -C userspace/

clean:
	rm -f *.o *.bin *~ init *.so
	$(MAKE) -C userspace clean
	$(MAKE) -C arch/boot_stage1 clean
	$(MAKE) -C src clean

%.o: %.c
	$(CC) $(KCCFLAGS) $< -c -o $@

src/%.kmo: src/**/*.rs src/*.rs src/*.c
	$(MAKE) -C src

stage2_entrypoint.o: stage2_entrypoint.s
	$(CC) $(KCFLAGS) -c stage2_entrypoint.s -o stage2_entrypoint.o

context_switch.o: context_switch.s
	$(CC) $(KCFLAGS) -c $< -o $@

trampoline.o: trampoline.s
	$(CC) $(KCFLAGS) -c $< -o $@

interrupthandler.o: interrupthandler.s include/cor/syscall.h intstubs.s~
	$(CC) $(KCFLAGS) -c -x assembler-with-cpp -Iinclude $< -o $@

test_mock_supplement.o: $(wildcard ./test_mock_supplement.c~) test_mock_supplement_stub.c
	if [ -e "test_mock_supplement.c~" ]; then f="test_mock_supplement.c~"; else f="test_mock_supplement_stub.c"; fi; $(CC) $(KCCFLAGS) -c -x c $$f -o $@

stage2.o: $(OBJS) linkerscript stage2_entrypoint.o src/libblock.a
	echo LONG\(0x$(shell git rev-parse HEAD | cut -c 1-6)\) > versionstamp~
	$(LD) $(OBJS) stage2_entrypoint.o src/lib.o -L./src -lblock -T linkerscript -o stage2.o --gc-sections -e stage2_entrypoint

stage2.bin: stage2.o
	$(OBJCOPY) --only-section=.text -O binary stage2.o stage2.bin

arch/boot_stage1/mbr.bin:
	$(MAKE) -C arch/boot_stage1

disk.bin: arch/boot_stage1/mbr.bin stage2.o Makefile
	# we want a 0x60100 byte disk
	dd if=/dev/zero of=disk.bin~ conv=notrunc bs=512 count=$$((0x60100 / 512))
	$(OBJCOPY) -I elf64-x86-64 --only-section=.text -O binary stage2.o stage2_text~
	$(OBJCOPY) -I elf64-x86-64 --only-section=.data -O binary stage2.o stage2_data~
	dd if=arch/boot_stage1/mbr.bin of=disk.bin~ conv=notrunc
	# add the text section
	dd if=stage2_text~ of=disk.bin~ conv=notrunc bs=512 seek=1
	# and add the data. ultra hacky to mess with the offsets here, and it's probably wrong, but meh...
	dd if=stage2_data~ of=disk.bin~ conv=notrunc bs=512 seek=$$((1 + (0x40000/512)))
	mv disk.bin~ disk.bin

intstubs.s~: Makefile
	ruby -e '0.upto(255) { |i| puts ".align 16\n.global intrstub_#{i}\nintrstub_#{i}:\n  push %rax\n  mov $$#{i}, %rax\n  jmp isr_dispatcher\n\n" }' > $@
