ROOT=..
include ../Makefile.conf
.PHONY: all clean

KMOS=block.kmo

all: $(KMOS)

clean:
	rm -f $(KMOS)

block.kmo: *.rs **/*.rs lib.c libkalloc/libkalloc.rlib
	rustc block.rs -C no-stack-check -C relocation-model=static -C code-model=large -L ./libkalloc
	gcc -o lib.o lib.c -c $(KCCFLAGS)
	gcc lib.o -o block.kmo~ -e rs_sched_exec -Wl,--gc-sections,-lm,-L.,-lblock,-r $(KCCFLAGS)
	objcopy block.kmo~ block.kmo -G rs_sched_exec -G virtio_init

libkalloc/libkalloc.rlib: libkalloc/*.rs
	rustc libkalloc/*.rs -C no-stack-check -C relocation-model=static -C code-model=large -O -o libkalloc/libkalloc.rlib
