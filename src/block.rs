#![crate_type="staticlib"]
#![feature(box_syntax,repr_simd)]
#![feature(alloc,collections,core_intrinsics,clone_from_slice)]
#![no_std]
#![feature(lang_items,unsafe_destructor,asm,box_patterns)]

// Use kalloc for heap memory
extern crate kalloc;

extern crate alloc;
#[macro_use(vec)]
extern crate collections;

// Set up the `print!` and `println!` macros, printing to the kernel console
#[macro_use]
mod print;
mod std { pub use core::fmt; } // std-module-trick to fix expansion of `format_args!`

mod usertask;
mod cpuio;
mod virtio;
mod fs;
mod kbuf;
mod mem;

pub mod sched;




/*let mut w = Vec::new();
write!(&mut w, "test");*/

static mut state : Option<State> = None;


extern {
  fn abort() -> !;
}

#[derive(Debug)]
struct State {
    number: u8,
    string: &'static str
}

extern "C" {
  fn pci_init();
  fn test_mock_main();
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

fn rust_pci_task() {
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
  sched::add_task(rust_pci_task, "PCI task");

  // Add a hook so we can insert things here when running tests.
  unsafe { test_mock_main(); }

  sched::add_task(usertask::exec_init, "init task");

  // Finally, we can pass control to all the tasks that have been set up by
  // now; the first task started will be run, and once it yields, the next
  // task registered with the scheduler will be started; eventually, we'll
  // round-robin between all the tasks.
  sched::exec();
}

fn thread1() {
  for i in 0..3 {
    println!("hi from thread1");
    sched::kyield();
  }
}

fn thread2() {
  for i in 0..3 {
    println!("hi from thread2");
    sched::kyield();
  }
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

#[inline(never)] #[cold]
fn print_panic_and_abort(msg: core::fmt::Arguments, file: &'static str, line: u32) -> ! {
    let mut kio = Kio;
    kio.write_fmt(format_args!("\npanic: "));
    kio.write_fmt(msg);
    kio.write_fmt(format_args!(" at {}:{}\n\n", file, line));
    unsafe { asm_abort(); }
}
