#!/bin/bash
set -eux
rm -f a.out

rustc mod_hello.rs -C no-stack-check -C relocation-model=static -L syscall.rs/target/
rm -fr x
mkdir x
mv libmod_hello.a x
(cd x && ar x libmod_hello.a)
#mv mod_hello.o dirty_mod_hello.o
#objcopy --rename-section _ZN9panicking5panic20hdb0bdef709ef82c5kwlE=rust_panicking__panic dirty_mod_hello.o mod_hello.o
gcc -o lib.o lib.c  -fdata-sections -ffunction-sections -Os -nostdlib -nostdinc -static -nostartfiles -nodefaultlibs -Wall -Wextra -m64 -Werror -std=c11 -fno-builtin
gcc -c entry.s -o entry.o
gcc x/*.o lib.o entry.o -Wl,-lm,--gc-sections -Os -nostdlib -nostdinc -static -nostartfiles -nodefaultlibs -Wall -Wextra -m64 -Werror -std=c11 -fno-builtin
#$(LD) $< dietlibc-0.33/bin-x86_64/start.o -Ldietlibc-0.33/bin-x86_64 -ldietc -o $@
(./a.out; echo $?); true
