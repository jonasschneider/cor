#![crate_type="staticlib"]
#![crate_name="cor"]
#![feature(box_syntax,repr_simd,const_fn,slice_bytes,fnbox)]
#![feature(alloc,collections,core_intrinsics,clone_from_slice,unboxed_closures)]
#![feature(lang_items,unsafe_destructor,asm,box_patterns,str_char,fn_traits)]
#![no_std]

extern crate kalloc;
extern crate alloc;

#[macro_use(vec)] // for `vec!`
extern crate collections;

mod prelude;

#[macro_use] // For `print!` and `println!`, writing to the kernel console
mod print;

mod byteorder;

mod usertask;
mod cpuio;
mod drivers;
mod fs;
mod kbuf;
mod mem;
mod sched;
mod sync;
mod block;

extern "C" {
  fn pci_init();
  fn asm_idle();
}

#[no_mangle]
pub fn virtio_init() {
  println!("virtio_init() called!");
}

#[no_mangle]
pub extern "C" fn handle_irq(irq: u8) {
  sched::irq::handle_irq(irq);
}

fn explore_pci() {
  unsafe { pci_init(); }
  println!("c-land pci_init exited");
}

fn idle_task() {
  loop {
    println!("Entering idle..");
    unsafe { asm_idle(); }
    println!("Exited idle.");
    sched::kyield();
  }
}

#[no_mangle]
pub fn rs_sched_exec() {
  sched::init();
  sched::add_task(idle_task, "idle");

  // Okay, now that we have the scheduler set up, we can start doing things
  // that set up tasks to react to input from the outside. A perfect example
  // is initializing PCI devices that occasionally send interrupts if they
  // have something to say to us.
  sched::add_task(explore_pci, "PCI task");

  sched::add_task(usertask::exec_init, "init task");

  // Finally, we can pass control to all the tasks that have been set up by
  // now; the first task started will be run, and once it yields, the next
  // task registered with the scheduler will be started; eventually, we'll
  // round-robin between all the tasks.
  sched::exec();
}

extern {
  fn asm_abort() -> !;
}

#[lang = "stack_exhausted"] extern fn stack_exhausted() {}
#[lang = "eh_personality"] extern fn eh_personality() {}

use collections::string::String;
use core::fmt::Write;
use print::Kio;

// It seems like the important thing is that we *don't* do stuff in here directly,
// because we might get inlined. Therefore, immediately do a call to somewhere
// that's marked as non-inlineable.
#[lang = "panic_fmt"]
extern fn handle_libcore_panic(msg: core::fmt::Arguments,
                                   file: &'static str, line: u32) -> ! {
  print_panic_and_abort(msg, file, line)
}

#[no_mangle]
pub extern "C" fn rust_panicmarker() {}

#[inline(never)] #[cold]
fn print_panic_and_abort(msg: core::fmt::Arguments, file: &'static str, line: u32) -> ! {
    rust_panicmarker();
    let mut kio = Kio;
    // ignore write errors and don't warn about them
    let _ = kio.write_fmt(format_args!("\npanic: "));
    let _ = kio.write_fmt(msg);
    let _ = kio.write_fmt(format_args!(" at {}:{}\n\n", file, line));
    unsafe { asm_abort(); }
}

#[no_mangle]
pub extern fn rs_init_interrupts() {
  sched::irq::init();
}
