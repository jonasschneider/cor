Cor: a hobbyist x86_64 kernel
=============================

Cor explores how one could build a bare-metal kernel in 2015.
It uses the [Rust](https://www.rust-lang.org/) programming language to achieve memory safety.

We find that the complexity of modern CPU architectures doesn't necessarily mean that you can't build nice things yourself.

Non-goals
---------
- Speed (though Rust is pretty fast out of the box)
- Security (you wouldn't attach this to your network at work)
- Production readiness

Synopsis
--------
- If you're not running Linux, it's easiest to grab [Vagrant](https://www.vagrantup.com/) and run `vagrant up`.
- Install dependencies: `build-essential`, qemu, xxd, Ruby, Go, Rust
- `$ make`
- `$ bin/run` to start the system, you'll be connected to the serial console of the machine
- `$ bin/debug` to debug the kernel, runs qemu drops you into a gdb
- `$ cucumber` to run integration/blackbox tests

- `$ bin/debug_stage1` to debug the bootloader (also see how `bin/debug` skips it)

Roadmap
-------
In the [OSDev ontology](http://wiki.osdev.org/What_order_should_I_make_things_in), we're likely building a ["Nick Stacky"](http://wiki.osdev.org/What_order_should_I_make_things_in#Nick_Stacky) system.

- [x] Boot *something*, and show a hello world on the Screen
- [x] Enter 64-bit long mode and never worry about the 80s again
- [x] Print something on the serial console
- [x] Set up debugging symbols & stack traces for stage2 kernel code
- [x] Switch to Multiboot/GRUB2 for reduced headaches
- [x] Minimal ELF userspace binary loader
  - [x] Make a minimal ELF that uses statically linked kernel functions to print something
  - [x] Make a minimal ELF that somehow signals that it's executing (HLT)
  - [x] Implement MVP ELF loader in stage2
  - [x] Load ELF into virtual memory
- [x] Implement syscall basics (choose INT 0x14 for fun, maybe start with just exit, then write)
- [x] permanent ring switch when starting init (except for syscalls)
- [x] Actual memory management & protection
  - [x] Read memory map from BIOS
  - [x] kalloc
  - [x] Dynamically allocate pages on ELF load
  - [x] sbrk
  - [x] Fix page permissions (`|4`s in boot.s)
  - [x] Move to higher-half kernel
- [x] Trampoline from C to Rust code after bootstrapping
- [x] Naive userspace page table setup for init
- [ ] Concurrency / Multiprocessing
  - [x] Enable PIT chip
  - [x] Cooperative scheduler for kernel tasks (using kyield)
  - [x] Idle task using HLT
  - [x] Basic system calls from userland
  - [ ] Yielding from userspace
  - [ ] Well-defined I/O blocking for kernel threads (runqueue, waitqueues)
  - [ ] Process memory space management (free everything on exit)
  - [ ] Process lifecycle / identity management, multiple processes, process table
  - [ ] Timer-based preemptive scheduling for userspace
  - [ ] fork()
  - [ ] Thread-local storage for user space
  - [ ] CPU-local storage for kernel space (for SMP: eliminate mutable statics)
- [x] Low-level non-spinning synchronization between kernel processes using wait/signal tokens
- [ ] Think harder about safe IRQ handling
  - strict separation between IRQ handler data and regular kernel data (Rust's ownership principles)
  - 'softirqs'/deferred processing for devices like virtio
  - context layers: userspace, kernelspace, irq? -> explicit synchronization always needed
  - special variable scopes: per-CPU, per-task, system-global (items, and the IRQ table)
- [ ] Filesystem
  - [x] Attach virtio (virtio-scsi, or preferredly virtio-blk) to QEMU
  - [x] PCI device detection
  - [x] virtio-blk block device driver
  - [x] no-op buffer page cache / buffer pool manager
  - [x] tiniest filesystem imaginable (read-only single-level?) -> `cpio` format
  - [x] read init from filesystem instead of baking it in
  - [ ] file descriptors / opening files from userspace
- [ ] Better toolchain for userspace
  - [x] Make a "hello world" binary that runs on host Linux and is as static as it gets (no libc)
  - [ ] Mod dietlibc to fit our syscall mechanism
  - [ ] Package that as libcorc or something
- [ ] Use an actual `i686-elf` cross-compiler instead of mingling with gcc-linux
- [ ] Build something like a shell that talks over serial (this will be our init)
- [ ] Userspace binaries (`ls` and such)
- [ ] Networking:
  - [ ] virtio NIC
  - [ ] Ethernet/Layer 2 broadcasting & receiving
  - [ ] DHCP, UDP with stub IP
  - [ ] real IP config
  - [ ] Ping
  - [ ] TCP
  - [ ] Tiny webserver
- [ ] Webserver in userspace
- [ ] SSH server userspace

- model I/O ports as slices of a 0-dimensional data type

More unicorns:

- [ ] Page-table-based IPC ("send" a page to another process, zero copy yadda yadda)
- [ ] SMP support
- [ ] Smarter kalloc (something like Linux' slab allocator?)
- [ ] Smarter userspace `malloc` that allocates contiguous sections for a single task
- [ ] FS: Journalling
- [ ] Test on real hardware
- [ ] Thread-local storage setup for Rustland:

    <nathan7> jonasschneider: but figure out some place to put your F-segment, and stick the address in the GDT
    <nathan7> jonasschneider: (and do an LGDT and all)
    <nathan7> jonasschneider: and mov 0, %fs:0x70 if you're lazy
    <nathan7> jonasschneider: or figure out some better place to put your stack

TODOs:

- compile with `-O`
  - correctly declare inline ASM memory barriers/volatility
- Fix relative addressing in `boot.s`
- Redzone thing



Memory map
----------
TODO: Make this entire map part of the linker script
Physical memory map at the time of stage2 startup:

- `0x01000-0x05FFF`: page tables courtesy of `boot.s`
- `0x6FFFF`: `0x37` (magic)
- `0x70000-0x7FFFF`: stage2's Stack (TODO: guard page)
- (I just realized that 0xa0000 - 0xfffff is still free, fuu base16)
- After that there are likely some memory holes

Additional virtual mapped memory:
- `0x0000008000000000-0x0000008000200000`: identity map of lower physical memory starting at `0`
  (this is where we keep & run the stage2 kernel)

Additional physical memory used by stage2:
- `0x06000-0x06FFF`: stage2's IDT (TODO: replace with kalloc)
- `0x81000` System timer jiffies counter (please don't ask)

All other memory is allocated in `mm.c` by `kalloc`, which uses the BIOS
 memory map provided by boot.s to place things into higher memory (usually,
 `phys >= 0x100000`.)

Caveats
-------
As this is an academic project, I'll try to document things I stumbled over.

- `%ax` is the same register as `%ah` and `%al`. That means: don't try to write something into `%ah`, then zero out `%ax` and expect your value in `%ah` to still be present.

- `gdb` doesn't handle QEMU architecture switches well. This can bite you when trying to debug the bootloader. I'm not yet sure what exactly breaks, but I've seen different failure modes when switching the CPU into 64-bit mode:
  1. `gdb` 7.6.2 on OS X (Homebrew) crashing after the switch, complaining about `g` packets. This seems to be a rather [known problem](http://www.cygwin.com/ml/gdb-patches/2012-03/msg00116.html). The linked thread also supplies a patch. Applying that leads us to symptom #2, which is:
  2. patched `gdb` 7.6.2 on OS X (Homebrew) *not* crashing after the switch, but still displaying the 32-bit registers, but with **wrong values**. This is apparently [also known](http://sourceware-org.1504.n7.nabble.com/Switching-architectures-from-a-remote-target-td111541.html), but is an issue with the `gdb` remote debugging protocol. (The QEMU monitor still displays the correct register values.)

  After debugging these, I realized that somehow Homebrew or OS X libs might be the culprit. And it turns out that under Linux (tested under Ubuntu and Arch), attaching to QEMU's `gdbserver` port *after* the switch to 64-bit mode works, but crashes when switching while attached. On the other hand, on OS X, the `g` packet crash happens even when attaching `gdb` after the switch to 64-bit mode.

  I'm not yet sure how to finally solve this. So far, the workaround seems to be to (a) run `gdb` under Linux, and (b) restart it when switching architectures. Meh.

- On Yosemite (not sure if relevant), `gdb`'s readline occasionally doesn't play nice with iTerm2. That means `gdb` will hang if it asks you a yes/no question, it won't respond to hitting the enter key after typing your answer. This happens both on a Homebrew-installed `gdb`, and over an SSH connection to an Ubuntu VM (via Vagrant). Terminal.app doesn't have this problem.

- QEMU does have some limited tracing support built-in. Running it with something like `-d int,pcall,cpu_reset,ioport,unimp,guest_errors` will spew various potentially helpful info to stderr. However, debugging generic errors like a General Protection fault still proves nontrivial. Using Homebrew's `interactive_shell` command in the qemu formula, qemu was patched to include some printf statements in the interrupt-handler code. This affects `do_interrupt64` (see `target-i386/seg_helper.c` in the qemu tree), for an example see [this gist](https://gist.github.com/315a19081f825583acf7)

- `info mem` in the qemu console will display the virtual memory map.

- Memory below `0x10000` cannot, in fact, belong to any segment, since segment 0 is the null segment. This, for some cases, means you can't have things in this low memory. An example seems to be the stack segment register when returning from an interrupt routine.

- Should maybe file a bug against QEMU because it doesn't check CS/SS register contents right when/somewhere shortly after entering protected mode, if you forget that it'll bite you later.

- The red zone thingie? (When interrupted in ring0)

- Relocation truncation https://www.technovelty.org/c/relocation-truncated-to-fit-wtf.html

- To investigate: ELF sizes --

      SECTIONS
      {
        . = 0x10000;
        .text : { *(.text) }
        . = 0x8000000;
        .data : { *(.data) }
        .bss : { *(.bss) }
      }

  is tiny, while swapping the addresses gives a huge one

- Design goal should probably "as little resident/permanent state in C-land as possible",
  given entropy and all that

- Continuity: The user space perspective is "do a syscall, then later *return* from the syscall", while the kernel has a completely different view.

- Context-switching idea: make the kernel-level scheduling, yielding, parking etc. independent of the trampoline/userspace/syscall/interrupt logic. It looks like they are orthogonal problems, at least when approached naively. For maximum performance, it's probably faster to mix everything.

The Story
---------
I don't have a history with writing anything low-level. I usually write Ruby or other dynamic languages with GC, and never really cared about what actually went down inside the computer. UNIX syscalls were my primitive instructions. `gdb` always scared me with its pointers, and how it could crash my entire process so easily. Finding `.s` files in a project repo was always a good sign for me to avoid touching it with a 10ft pole.

Takeaway: For high-level developers, the scare factor of low-level assembly programming might be so high because it's combined with the great complexity of a modern OS. If you take one of the factors away, you're back in a fairly comfortable zone; usually, you take away the low-level factor and deal with the complexity. It turns out that taking away the complexity works just as well. (Difficulty = Complexity x Scope)


Bibliography
------------
- http://wiki.osdev.org/Memory_Map_(x86)
- Intel manual (TODO)
- AMD manual (TODO)
- http://idak.gop.edu.tr/esmeray/UnderStandingKernel.pdf


Lessons learned
--------------
- **Ownership is a powerful concept of resource management.** Case study: CPU I/O ports.
  1. If you are able to access a port, nobody else can (=unique owner)
  2. You can temporarily give somebody else access, but during that time, you don't have access yourself (=borrowing)
  3. You can give away access to a *part* of a port (=slice splitting)
- Linux `ops` structs do dynamic dispatch much like `vtables`
- Rust is great at moving on the ladder of abstraction. (Generics/Traits vs inline asm)
