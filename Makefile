ROOT=.
include Makefile.conf
.PHONY: all clean

all:
	$(MAKE) -C arch/x86-multiboot/
	$(MAKE) -C userspace/

clean:
	rm -fr target
	$(MAKE) -C arch/x86-multiboot/ clean
	$(MAKE) -C userspace/ clean
