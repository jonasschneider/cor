ROOT=..
include ../Makefile.conf
.PHONY: all clean

all: libblock.a lib.o

clean:
	rm -f libblock.a libkalloc/libkalloc.rlib lib.o

lib.o: lib.c
	gcc -o lib.o lib.c -c $(KCCFLAGS)

libblock.a: *.rs **/*.rs libkalloc/libkalloc.rlib
	rustc block.rs -C no-stack-check -C relocation-model=static -C code-model=large -L ./libkalloc

libkalloc/libkalloc.rlib: libkalloc/*.rs
	rustc libkalloc/*.rs -C no-stack-check -C relocation-model=static -C code-model=large -O -o libkalloc/libkalloc.rlib
