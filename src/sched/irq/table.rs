use super::UnsafeTableEntry;
use alloc::boxed::Box;
use collections::vec::Vec;
use core::fmt;

use sync::global_mutex::GlobalMutex;

// This is where the magic happens. This table is shared between interrupt handlers and
// "normal" kernel space, so we need to make sure that we appropriately lock it.
// TODO: this only works for the `critical` part (nesting IRQs will cause a deadlock),
// figure out what to do here.
// TODO: This could be a GlobalRWLock instead of the heavier GlobalMutex...right?
// Well, only if ISRs don't need mutable access to their data. Which I don't think is true.
// A better solution using UnsafeCell is described here:
// https://internals.rust-lang.org/t/pre-rfc-remove-static-mut/1437
static mut TABLE: Option<*mut [UnsafeTableEntry]> = None;

pub fn init() {
  if let Some(_) = unsafe { TABLE } {
    panic!("TABLE already set up!");
  }

  let s = &UnsafeTableEntry::new() as &Sync;
  let mut tab: Vec<UnsafeTableEntry> = Vec::with_capacity(256);
  for _ in 0..256 {
    tab.push(UnsafeTableEntry::new());
  }

  unsafe {
    TABLE = Some(Box::into_raw(tab.into_boxed_slice()));
    // will "leak" the data into TABLE
  }
}

// TODO: should we even be able to print from IRQ-land?
// this is like __do_IRQ in Linux
pub fn handle_irq(num: u8) {
  print!("\x1B[;31m");
  println!("[[[Bang! IRQ 0x{:x} handled by sched::irq",num);

  let mut entry = unsafe { &mut (*(TABLE.unwrap()))[num as usize] };

  entry.trigger(num);
  println!("sched::irq is done with interrupt 0x{:x}]]]",num);
  print!("\x1B[0m");
}
