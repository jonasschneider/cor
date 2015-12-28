ROOT=../../..
include ../../../Makefile.conf
.PHONY: all clean

OBJS=main.o printk.o chrdev_serial.o chrdev_console.o io.o interrupthandler.o tss.o mm.o
OBJS+=task.o pci.o timer.o pic.o interrupt.o
OBJS+=context_switch.o trampoline.o idle.o

all: $(OBJS)

clean:
	rm -f $(OBJS)

%.o: %.c
	$(CC) $(KCCFLAGS) $< -c -o $@

context_switch.o: context_switch.s
	$(CC) $(KCFLAGS) -c $< -o $@

trampoline.o: trampoline.s
	$(CC) $(KCFLAGS) -c $< -o $@

interrupthandler.o: interrupthandler.s ../../../include/cor/syscall.h intstubs.s~
	$(CC) $(KCFLAGS) -c -x assembler-with-cpp -I../../../include $< -o $@

intstubs.s~: Makefile
	ruby -e '0.upto(255) { |i| puts ".align 16\n.global intrstub_#{i}\nintrstub_#{i}:\n  push %rax\n  mov $$#{i}, %rax\n  jmp isr_dispatcher\n\n" }' > $@
