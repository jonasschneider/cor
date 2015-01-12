ROOT=..
include ../Makefile.conf
.PHONY: all clean

KMOS=block.kmo

all: $(KMOS)

clean:
	rm -f $(KMOS)

block.kmo: *.rs **/*.rs lib.c
	rustc block.rs -C no-stack-check -C relocation-model=static -g -C code-model=large
	gcc -o lib.o lib.c -c $(KCCFLAGS)
	gcc lib.o -o block.kmo~ -e rs_sched_exec -Wl,--gc-sections,-lm,-L.,-lblock,-r $(KCCFLAGS)
	objcopy block.kmo~ block.kmo -N _GLOBAL_OFFSET_TABLE_ -G rs_sched_exec # mark all except the global entry point as local
