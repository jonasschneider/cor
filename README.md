Physical memory map at the time of stage2 startup

- `0x10000-0x4FFFF`: Kernel `.text`, `0x10000` is the x86_64 entry point from bootloader
- `0x50000-0x6FFFD`: Kernel `.data` and stack, not sure yet if this is a good idea
- `0x6FFFE`: `0x13` (magic)
- `0x6FFFF`: `0x37` (magic)

Synopsis
=========
- `make`
- `bin/run` to launch the thing in qemu
- To debug the kernel, `bin/debug` and then attach a `gdb` like so:

    set disassemble-next-line on
    set arch i386:x86-64
    tar rem :1234

- To debug the bootloader, look at how `bin/debug` skips it

The tools
=========
- `qemu` is a great emulator. It has an intuitive CLI, does what you'd expect, and even has a nice debugging console.
- The GNU `binutils` provide great insight into what actually comes out of your compiler/assembler. `objcopy`, `objdump` allowed me to actually mess with the compilation, disassemble things, and shuffle sections around to form the actual boot media.
- The almighty `gdb`

Caveats
=======
As this is an academic project, I'll try to document things I stumbled over.

- `%ax` is the same register as `%ah` and `%al`. That means: don't try to write something into `%ah`, then zero out `%ax` and expect your value in `%ah` to still be present.

- `gdb` doesn't handle QEMU architecture switches well. This can bite you when trying to debug the bootloader. I'm not yet sure what exactly breaks, but I've seen different failure modes when switching the CPU into 64-bit mode:
  1. `gdb` 7.6.2 on OS X (Homebrew) crashing after the switch, complaining about `g` packets. This seems to be a rather [known problem](http://www.cygwin.com/ml/gdb-patches/2012-03/msg00116.html). The linked thread also supplies a patch. Applying that leads us to symptom #2, which is:
  2. patched `gdb` 7.6.2 on OS X (Homebrew) *not* crashing after the switch, but still displaying the 32-bit registers, but with **wrong values**. This is apparently [also known](http://sourceware-org.1504.n7.nabble.com/Switching-architectures-from-a-remote-target-td111541.html), but is an issue with the `gdb` remote debugging protocol. (The QEMU monitor still displays the correct register values.)

  After debugging these, I realized that somehow Homebrew or OS X libs might be the culprit. And it turns out that under Linux (tested under Ubuntu and Arch), attaching to QEMU's `gdbserver` port *after* the switch to 64-bit mode works, but crashes when switching while attached. On the other hand, on OS X, the `g` packet crash happens even when attaching `gdb` after the switch to 64-bit mode.

  I'm not yet sure how to finally solve this. So far, the workaround seems to be to (a) run `gdb` under Linux, and (b) restart it when switching architectures. Meh.


History
=======
I don't have a long history with writing anything low-level. I usually program in Ruby or other dynamic languages with GC, and never really cared about what actually went down inside the computer. UNIX syscalls were my primitive instructions. `gdb` always scared me with its pointers, and how it could crash my entire process so easily. Finding `.S` files in a project repo was always a good sign for me to avoid touching anything with a 200ft pole.

Takeaway: For high-level developers like me, the scare factor of low-level assembly programming might be so high because it's combined with the great complexity of a modern OS. If you take one of the factors away, you're back in a fairly comfortable zone; usually, you take away the low-level factor and deal with the complexity. It turns out that taking away the complexity works just as well.
