mod table;

// Global should both lock for IRQs and other CPUs, and as such is the only
// stuff that shuld be placed in static memory.
// -> We have to make sure that only Sync things are accessible from both interrupts and normal kernel code.

// locking a sleeping lock sync borrows ownership of the process context
// -> enforces that sleeping mutexes can only be acquired in process context

use alloc::boxed::Box;
use collections::vec::Vec;
use core::fmt;

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
#[derive(Debug)]
struct TableEntry {
  handlers: Vec<Box<InterruptHandler>>,
}

impl TableEntry {
  pub fn trigger(&mut self, num: u8) {
    println!("Triggering: {:?}", self);
    if num == 0x30 {
      println!("Is early test interrupt, OK.");
      return;
    }
  }
}

pub fn init() {
  table::init();
}

pub fn handle_irq(num: u8) {
  table::handle_irq(num);
}
