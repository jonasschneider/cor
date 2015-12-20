// types of lock disciplines: Mutex, RWLock
// types of blocking: Sleep(=Semaphore), Spin
// types of lock scopes: Global, ..?

// Global should both lock for IRQs and other CPUs, and as such is the only
// stuff that shuld be placed in static memory.
// -> We have to make sure that only Sync things are accessible from both interrupts and normal kernel code.

// locking a sleeping lock sync borrows ownership of the process context
// -> enforces that sleeping mutexes can only be acquired in process context

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
// TODO: This is an Option because I don't want to initialize the mutex at startup.
// Maybe with `Default` we can have a default-unlocked mutex?
// TODO: The Option actually allows a race.
// https://github.com/rust-lang-nursery/lazy-static.rs/blob/master/src/lib.rs might be able to fix it
// TODO: This could be a GlobalRWLock instead of the heavier GlobalMutex...right?
// Well, only if ISRs don't need mutable access to their data. Which I don't think is true.
// The problem is: Rust won't let us store the actual thing in here, because of initialization sync issues.
// We can't use Table, Box<Table>, or Option<Table>, and neither *mut Table (since it's not Sync).
// Sooo... we have to cheat. :(
// This is safe as long as we set up interrupts correctly: init() must be called (to set up the Table)
// before handle_irq() is ever called (to read from the Table). TODO: if Table is 0 in the latter, we should
// explicitly scream.
type Table = [GlobalMutex<TableEntry>; 256];
//static IRQTABLE: *mut Table = 0 as *mut Table;
static mut IRQTABLE: *mut u8 = 0 as *mut u8;

use alloc::boxed::Box;
use collections::vec::Vec;
use core::fmt;

type TableDummy = u8;

trait InterruptHandler {
  fn critical(&mut self); // will be executed with interrupts disabled
  fn noncritical(&self); // will be executed with interrupts disabled, ISR shared
}
impl fmt::Debug for Box<InterruptHandler + Sync> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<handler>")
    }
}

// Comparable to irq_desc_t (Table 4-4 in UTLK)
#[derive(Debug)]
enum TableEntry {
  Uninitialized,
  Handlers(Vec<Box<InterruptHandler + Sync>>),
}

// This is super ugly and will bite us if we, at any point, want to modify/replace the table
pub fn init() {
  // AUDIT THIS PLEASE
  if unsafe {IRQTABLE != (0 as *mut u8)} {
    panic!("IRQTABLE already set up!");
  }
  unsafe {
    let mut tab: Vec<GlobalMutex<TableEntry>> = Vec::with_capacity(256);
    for _ in 0..256 {
      tab.push(GlobalMutex::new(TableEntry::Uninitialized));
    }
    IRQTABLE = Box::into_raw(tab.into_boxed_slice()) as *mut u8;
    // will "leak" tab, but into the global, so we're fine
  }
}

// TODO: should we even be able to print from IRQ-land?
pub fn handle_irq(num: u8) {
  println!("[Bang! IRQ 0x{:x}]",num);

  // START AUDIT HERE
  if unsafe { IRQTABLE == (0 as *mut u8) } {
    println!("WARN: You didn't set up IRQTABLE. I'll do it for you now, but this is just asking for trouble.");
    init(); // FIXME: no, just no
    //panic!("You didn't set up IRQTABLE correctly.");
  }
  let mut entry_mutex = unsafe { &mut (*(IRQTABLE as *mut Table))[num as usize] };
  // END AUDIT HERE :)
  let entry = entry_mutex.lock(); // lock it down; this could have finer granularity

  println!("IRQTABLE entry: {:?}", entry);
  if num == 0x30 {
    println!("Is early test interrupt, OK.");
    return;
  }
  match entry {
    Uninitialized => {
      panic!("Entry is uninitialized, so I didn't expect to receive this IRQ.");
    }
  }
}
