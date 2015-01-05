ROOT=..
include ../Makefile.conf
.PHONY: all clean

KMOS=hello.kmo

all: $(KMOS)

clean:
	rm -f $(KMOS)

hello.kmo: mod_hello.rs
	rustc mod_hello.rs -C no-stack-check -C relocation-model=static -g
	gcc -o lib.o lib.c -c -I../include $(KCCFLAGS)
	ar rcs libtinyc.a lib.o
	gcc -c entry.s -o entry.o -ggdb $(KCFLAGS)
	gcc -o hello.kmo~ -e hello_main -Wl,--gc-sections,-lm,-r,-L.,-ltinyc,-lmod_hello $(KCCFLAGS)
	objcopy hello.kmo~ hello.kmo -G hello_main # strip all except the global entry point
