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

use collections::vec::Vec;

type InterruptHandler = fn();

enum TableEntry {
  Uninitialized,
  Handlers(Vec<InterruptHandler>),
}

//static IRQTABLE: [GlobalMutex<TableEntry>; 256] = ; // TODO: this could be a (faster) RWLock

pub fn handle_irq(num: u8) {
  println!("[Whoop: IRQ 0x{:x}]",num);
}
