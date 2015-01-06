ROOT=..
include ../Makefile.conf
.PHONY: all clean

KMOS=hello.kmo block.kmo

all: $(KMOS)

clean:
	rm -f $(KMOS)

hello.kmo: mod_hello.rs lib.c
	rustc mod_hello.rs -C no-stack-check -C relocation-model=static -g --cfg external_funcs
	gcc -o lib.o lib.c -c $(KCCFLAGS)
	gcc lib.o -o hello.kmo~ -e hello_main -Wl,--gc-sections,-lm,-r,-L.,-lmod_hello $(KCCFLAGS)
	objcopy hello.kmo~ hello.kmo -N _GLOBAL_OFFSET_TABLE_ -G hello_main # mark all except the global entry point as local

block.kmo: block.rs lib.c
	rustc block.rs -C no-stack-check -C relocation-model=static -g --cfg external_funcs -C code-model=large
	gcc -o lib.o lib.c -c $(KCCFLAGS)
	gcc lib.o -o block.kmo~ -e virtio_init -Wl,--gc-sections,-lm,-L.,-lblock,-r $(KCCFLAGS)
	objcopy block.kmo~ block.kmo -N _GLOBAL_OFFSET_TABLE_ -G virtio_init # mark all except the global entry point as local
