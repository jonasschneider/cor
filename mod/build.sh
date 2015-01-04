#!/bin/bash
set -ux

if [ ! -d syscall.rs ]; then
    git clone https://github.com/kmcallister/syscall.rs
    (cd syscall.rs && cargo build)
    echo
fi

rustc mod_hello.rs -C no-stack-check -C relocation-model=static -L syscall.rs/target/
ar x libmod_hello.rlib mod_hello.o
objcopy -N _start mod_hello.o cool_mod_hello.o
gcc -nostdlib -nostdinc -static -nostartfiles -nodefaultlibs -Wall -Wextra -m64 -Werror -std=c11 -fno-builtin -ggdb -o lib.o lib.c
gcc -c entry.s -o entry.o
ld --gc-sections -e_start mod_hello.o entry.o lib.o

./a.out

echo $?
