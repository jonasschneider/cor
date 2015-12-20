use super::TableEntry;
use alloc::boxed::Box;
use collections::vec::Vec;
use core::fmt;

// types of lock disciplines: Mutex, RWLock
// types of blocking: Sleep(=Semaphore), Spin
// types of lock scopes: Global, ..?

// 1. spinlock makes sure that nobody except us can enter (not even in interrupt etc.)
// 2. however, interrupts create a deadlock condition: while kernel has spinlock, irq on same CPU spins for lock -> dead
// 3. solution: disable local interrupts *before* acquiring the lock to prevent the deadlock condition
//    (the lock is still needed to protect against accesses by other CPUs, which we don't really care about yet)
// -> GlobalMutex = CLI + Spinlock

struct GlobalMutex<T> {
  inner: T
}
impl<T> GlobalMutex<T> {
  fn new(x: T) -> Self {
    GlobalMutex { inner: x }
  }
  fn lock<'t>(&'t mut self) -> &'t mut T {
    // FIXME: this is not actually doing anything
    &mut self.inner
  }
}

// This is where the magic happens. This table is shared between interrupt handlers and
// "normal" kernel space, so we need to make sure that we appropriately lock it.
// TODO: this only works for the `critical` part (nesting IRQs will cause a deadlock),
// figure out what to do here.
// TODO: This could be a GlobalRWLock instead of the heavier GlobalMutex...right?
// Well, only if ISRs don't need mutable access to their data. Which I don't think is true.
// A better solution using UnsafeCell is described here:
// https://internals.rust-lang.org/t/pre-rfc-remove-static-mut/1437
type Table = [GlobalMutex<TableEntry>];
static mut TABLE: Option<*mut Table> = None;

pub fn init() {
  if let Some(s) = unsafe { TABLE } {
    panic!("TABLE already set up!");
  }

  let mut tab: Vec<GlobalMutex<TableEntry>> = Vec::with_capacity(256);
  for _ in 0..256 {
    tab.push(GlobalMutex::new(TableEntry{ handlers: Vec::new() }));
  }

  unsafe {
    TABLE = Some(Box::into_raw(tab.into_boxed_slice()));
    // will "leak" the data into TABLE
  }
}

// TODO: should we even be able to print from IRQ-land?
pub fn handle_irq(num: u8) {
  print!("\x1B[;31m");
  println!("[[[Bang! IRQ 0x{:x} handled by sched::irq::table",num);

  let mut entry_mutex = unsafe { &mut (*(TABLE.unwrap()))[num as usize] };
  let entry = entry_mutex.lock(); // lock it down; this could have finer granularity

  entry.trigger(num);
  println!("sched::irq::table is done with interrupt 0x{:x}]]]",num);
  print!("\x1B[0m");
}
