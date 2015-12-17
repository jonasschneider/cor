#![crate_type="staticlib"]
#![feature(box_syntax)]
#![feature(alloc,collections,core_intrinsics)]
#![no_std]
#![feature(lang_items,unsafe_destructor,asm,box_patterns)]

// only for bindgen'd stuff!
#![feature(libc)]

// Use kalloc for heap memory
extern crate kalloc;

extern crate alloc;
#[macro_use(vec)]
extern crate collections;

// Set up the `print!` and `println!` macros, printing to the kernel console
#[macro_use]
mod print;
mod std { pub use core::fmt; } // std-module-trick to fix expansion of `format_args!`

// cpuio library
mod cpuio;

// import submodules
mod virtio;

mod kbuf;

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
  fn init_task();
  fn pci_init();
  fn test_mock_main();
}

fn rust_init_task() {
  unsafe { init_task(); }
  println!("c-land init_task exited.?!, loop-yielding");
  while(true) { sched::kyield(); }
}

fn rust_pci_task() {
  unsafe { pci_init(); }
  println!("c-land pci_init exited");
}

static mut called : bool = false;

#[no_mangle]
pub fn rs_sched_exec() {
  unsafe {
    if called {
      sched::kyield();
      return;
    }
    called = true;
  }
  unsafe { sched::init(); }
  sched::add_task(thread1, "thread1");
  sched::add_task(thread2, "thread2");

  // Okay, now that we have the scheduler set up, we can start doing things
  // that set up tasks to react to input from the outside. A perfect example
  // is initializing PCI devices that occasionally send interrupts if they
  // have something to say to us.
  sched::add_task(rust_pci_task, "PCI task");

  // Add a hook so we can insert things here when running tests.
  unsafe { test_mock_main(); }

  // we cannot yet yield back from the init task, so don't start it
  //sched::add_task(rust_init_task, "init task");

  // Finally, we can pass control to all the tasks that have been set up by
  // now; the first task started will be run, and once it yields, the next
  // task registered with the scheduler will be started; eventually, we'll
  // round-robin between all the tasks.
  unsafe { sched::exec(); }
}

fn thread1() {
  while(true) {
    println!("hi from thread1");
    sched::kyield();
  }
}

fn thread2() {
  while(true) {
    println!("hi from thread2");
    sched::kyield();
  }
}


#[no_mangle]
pub fn virtio_init(ioport : u16) {
  // apparently, anything modifying global mutable state is considered unsafe...
  unsafe { state = Some(State { number: 1337, string: "" }); }

  // okay, even reading requires it
  unsafe {
    match state {
      Some(ref mut s) => s.number = 1338,
      None => (),
    }
  }

  unsafe { println!("the state is now {:?}, lol", state); }
  let a_device = unsafe { virtio::init(ioport) };
  println!("result of blockdevice init: {:?}", a_device);
}

#[lang = "stack_exhausted"] extern fn stack_exhausted() {}
#[lang = "eh_personality"] extern fn eh_personality() {}
#[lang = "panic_fmt"] fn panic_fmt() -> ! { loop {} }
