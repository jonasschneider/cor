ROOT=../..
include ../../Makefile.conf
.PHONY: clean

all: boot.o

boot.o: boot.s
	$(AS) boot.s -o boot.o

clean:
	rm -f boot.o
