ROOT=..
include ../Makefile.conf
.PHONY: all clean

all: libcor.a lib.o

clean:
	rm -f libcor.a libkalloc.rlib lib.o

lib.o: lib.c
	gcc -o lib.o lib.c -c $(KCCFLAGS)

libcor.a: *.rs **/*.rs libkalloc.rlib
	rustc lib.rs -C no-stack-check -C relocation-model=static -C code-model=large -L .

libkalloc.rlib: libkalloc/*.rs
	rustc libkalloc/*.rs -C no-stack-check -C relocation-model=static -C code-model=large -O -o libkalloc.rlib
