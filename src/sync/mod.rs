// types of lock disciplines: Mutex, RWLock
// types of blocking: Sleep(=Semaphore), Spin
// types of lock scopes: Global, Sleeping(=only for kernel tasks, not interrupt handlers), others?


// 1. spinlock makes sure that nobody except us can enter (not even in interrupt etc.)
// 2. however, interrupts create a deadlock condition: while kernel has spinlock, irq on same CPU spins for lock -> dead
// 3. solution: disable local interrupts *before* acquiring the lock to prevent the deadlock condition
//    (the lock is still needed to protect against accesses by other CPUs, which we don't really care about yet)
// -> GlobalMutex = CLI + Spinlock
pub mod global_mutex;
