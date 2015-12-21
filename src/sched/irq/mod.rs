mod table;
use sync::global_mutex::GlobalMutex;

// Global should both lock for IRQs and other CPUs, and as such is the only
// stuff that shuld be placed in static memory.
// -> We have to make sure that only Sync things are accessible from both interrupts and normal kernel code.

// locking a sleeping lock sync borrows ownership of the process context
// -> enforces that sleeping mutexes can only be acquired in process context

use alloc::boxed::Box;
use collections::vec::Vec;
use core::fmt;
use core::sync::atomic::{AtomicBool,Ordering};

trait InterruptHandler {
  fn critical(&mut self); // will be executed with interrupts disabled
  fn noncritical(&self); // will be executed with interrupts disabled, ISR shared
}
impl fmt::Debug for Box<InterruptHandler> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<handler>")
    }
}

// Comparable to irq_desc_t (Table 4-4 in UTLK)
// THIS HAS TO BE SYNC! Unsafe because we're not statically enforcing the Sync-ness
#[derive(Debug)]
struct UnsafeTableEntry {
  again: AtomicBool,
  handlers: GlobalMutex<Vec<Box<InterruptHandler>>>,
}

impl UnsafeTableEntry {
  pub fn new() -> Self {
    UnsafeTableEntry {
      again: AtomicBool::new(false),
      handlers: GlobalMutex::new(vec![]),
    }
  }
  pub fn trigger(&mut self, num: u8) {
    // This is pretty much like Linux's IRQ_PENDING bit.
    // See the UTLK section about __do_IRQ for more info.

    // I think we can do Relaxed since other reads/writes are protected by the mutex.
    self.again.store(true, Ordering::Relaxed);

    let handlers = match self.handlers.try_lock() {
      None => {
        println!("Somebody else has the handler lock. Quitting.");
        return;
      },
      Some(e) => e
    };
    println!("Triggering handlers: {:?}", *handlers);
    if handlers.len() == 0 {
      if num == 0x30 {
        println!("Is early test interrupt, OK.");
        return;
      } else {
        println!("No handlers found, this interrupt is unexpected. Wut?");
        panic!("strange interrupt");
      }
    }
  }
}

pub fn init() {
  table::init();
}

pub fn handle_irq(num: u8) {
  table::handle_irq(num);
}
