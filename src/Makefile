ROOT=..
include ../Makefile.conf
.PHONY: all clean

KMOS=block.kmo

all: $(KMOS)

clean:
	rm -f $(KMOS)

block.kmo: *.rs **/*.rs lib.c libkalloc/libkalloc.rlib
	rustc block.rs -C no-stack-check -C relocation-model=static -C code-model=large -O -L ./libkalloc
	gcc -o lib.o lib.c -c $(KCCFLAGS)
	gcc lib.o -o block.kmo~ -e rs_sched_exec -Wl,-lm,-L.,-lblock,-r $(KCCFLAGS)
	objcopy block.kmo~ block.kmo -N _GLOBAL_OFFSET_TABLE_ -G rs_sched_exec # mark all except the global entry point as local

libkalloc/libkalloc.rlib: libkalloc/*.rs
	rustc libkalloc/*.rs -C no-stack-check -C relocation-model=static -C code-model=large -O -o libkalloc/libkalloc.rlib
