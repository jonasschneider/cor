ROOT=..
include ../Makefile.conf
.PHONY: all clean

KMOS=hello.kmo

all: $(KMOS)

clean:
	rm -f $(KMOS)

hello.kmo: mod_hello.rs lib.c
	rustc mod_hello.rs -C no-stack-check -C relocation-model=static -g
	gcc -o lib.o lib.c -c -I../include $(KCCFLAGS)
	gcc lib.o -o hello.kmo~ -e hello_main -Wl,--gc-sections,-lm,-r,-L.,-lmod_hello $(KCCFLAGS)
	objcopy hello.kmo~ hello.kmo -N _GLOBAL_OFFSET_TABLE_ -G hello_main # mark all except the global entry point as local
