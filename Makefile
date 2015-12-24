ROOT=.
include Makefile.conf
.PHONY: all clean

OBJS=main.o printk.o chrdev_serial.o chrdev_console.o io.o interrupthandler.o tss.o mm.o task.o pci.o timer.o pic.o interrupt.o
OBJS+=context_switch.o trampoline.o idle.o

all:
	$(MAKE) -C src/
	$(MAKE) cor.iso
	$(MAKE) -C userspace/

clean:
	rm -f *.o *.bin *~ init *.so
	$(MAKE) -C userspace clean
	$(MAKE) -C arch/boot_stage1 clean
	$(MAKE) -C src clean

%.o: %.c
	$(CC) $(KCCFLAGS) $< -c -o $@

stage2_entrypoint.o: stage2_entrypoint.s
	$(CC) $(KCFLAGS) -c stage2_entrypoint.s -o stage2_entrypoint.o

context_switch.o: context_switch.s
	$(CC) $(KCFLAGS) -c $< -o $@

trampoline.o: trampoline.s
	$(CC) $(KCFLAGS) -c $< -o $@

interrupthandler.o: interrupthandler.s include/cor/syscall.h intstubs.s~
	$(CC) $(KCFLAGS) -c -x assembler-with-cpp -Iinclude $< -o $@

cor.iso: cor.elf
	cp cor.elf arch/boot_multiboot/iso/boot/cor.elf && grub-mkrescue -o $@ arch/boot_multiboot/iso

cor.elf: $(OBJS) src/lib.o Makefile arch/boot_multiboot/boot.o arch/boot_multiboot/multiboot.ld
	echo LONG\(0x$(shell git rev-parse HEAD | cut -c 1-6)\) > versionstamp~
	gcc -mcmodel=large -Wl,-n,--build-id=none -ffreestanding -O2 -nostdlib -lgcc -o $@ $(OBJS) src/lib.o -L./src -lcor arch/boot_multiboot/boot.o -T arch/boot_multiboot/multiboot.ld

intstubs.s~: Makefile
	ruby -e '0.upto(255) { |i| puts ".align 16\n.global intrstub_#{i}\nintrstub_#{i}:\n  push %rax\n  mov $$#{i}, %rax\n  jmp isr_dispatcher\n\n" }' > $@
