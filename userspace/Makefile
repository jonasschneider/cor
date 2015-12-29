ROOT=..
include ../Makefile.conf
.PHONY: all clean

all:
	$(MAKE) -C ash
	$(MAKE) rootfs.bin

clean:
	$(MAKE) -C ash clean
	rm -fr dietlibc-0.33{,.tar.bz2} init init.ld rootfs.bin

rootfs.bin: init Makefile
	(echo init) | cpio --create > $@

init: init.c init_lib.c init.ld ../include/cor/*.h ash/ash
	#cp ash/ash init
	$(CC) $(CCFLAGS) -include stdlib.h init.c init_lib.c -T init.ld -o init

crt0.o: init_lib.c
	$(CC) $(CCFLAGS) -c init_lib.c -o $@

init.ld: init.ld_default
	cp $< $@

# date.o: date.c dietlibc-0.33/include
# 	$(CC) $(CFLAGS) date.c -o date.o

# date: date.o dietlibc-0.33/bin-x86_64
# 	$(LD) $< dietlibc-0.33/bin-x86_64/start.o -Ldietlibc-0.33/bin-x86_64 -ldietc -o $@

dietlibc-0.33/bin-x86_64: dietlibc-0.33
	cd dietlibc-0.33 && make && mv bin-x86_64/{dietlibc.a,libdietc.a}

dietlibc-0.33/include: dietlibc-0.33
dietlibc-0.33:
	wget http://www.fefe.de/dietlibc/dietlibc-0.33.tar.bz2
	shasum -c dietlibc-0.33.tar.bz2.sha1
	tar xjf dietlibc-0.33.tar.bz2
