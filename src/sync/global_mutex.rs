// from https://github.com/mvdnes/spinlock-rs/blob/master/src/mutex.rs
// FIXME: actually make it global by disabling interrupts (using depth like linux?)

use core::sync::atomic::{AtomicBool, Ordering, ATOMIC_BOOL_INIT};
use core::cell::UnsafeCell;
use core::marker::Sync;
use core::ops::{Drop, Deref, DerefMut};
use core::fmt;
use core::option::Option::{self, None, Some};
use core::default::Default;

/// This type provides MUTual EXclusion based on spinning.
///
/// # Description
///
/// This structure behaves a lot like a normal GlobalMutex. There are some differences:
///
/// - It may be used outside the runtime.
///   - A normal GlobalMutex will fail when used without the runtime, this will just lock
///   - When the runtime is present, it will call the deschedule function when appropriate
/// - No lock poisoning. When a fail occurs when the lock is held, no guarantees are made
///
/// When calling rust functions from bare threads, such as C `pthread`s, this lock will be very
/// helpful. In other cases however, you are encouraged to use the locks from the standard
/// library.
///
/// # Simple example
///
/// ```
/// use spin;
/// let spin_GlobalMutex = spin::GlobalMutex::new(0);
///
/// // Modify the data
/// {
///     let mut data = spin_GlobalMutex.lock();
///     *data = 2;
/// }
///
/// // Read the data
/// let answer =
/// {
///     let data = spin_GlobalMutex.lock();
///     *data
/// };
///
/// assert_eq!(answer, 2);
/// ```
///
/// # Thread-safety example
///
/// ```
/// use spin;
/// use std::sync::{Arc, Barrier};
///
/// let numthreads = 1000;
/// let spin_GlobalMutex = Arc::new(spin::GlobalMutex::new(0));
///
/// // We use a barrier to ensure the readout happens after all writing
/// let barrier = Arc::new(Barrier::new(numthreads + 1));
///
/// for _ in (0..numthreads)
/// {
///     let my_barrier = barrier.clone();
///     let my_lock = spin_GlobalMutex.clone();
///     std::thread::spawn(move||
///     {
///         let mut guard = my_lock.lock();
///         *guard += 1;
///
///         // Release the lock to prevent a deadlock
///         drop(guard);
///         my_barrier.wait();
///     });
/// }
///
/// barrier.wait();
///
/// let answer = { *spin_GlobalMutex.lock() };
/// assert_eq!(answer, numthreads);
/// ```
pub struct GlobalMutex<T>
{
    lock: AtomicBool,
    data: UnsafeCell<T>,
}

/// A guard to which the protected data can be accessed
///
/// When the guard falls out of scope it will release the lock.
pub struct GlobalMutexGuard<'a, T:'a>
{
    lock: &'a AtomicBool,
    data: &'a mut T,
}

unsafe impl<T> Sync for GlobalMutex<T> {}

impl<T> GlobalMutex<T>
{
    /// Creates a new spinlock wrapping the supplied data.
    ///
    /// May be used statically:
    ///
    /// ```
    /// #![feature(const_fn)]
    /// use spin;
    ///
    /// static GlobalMutex: spin::GlobalMutex<()> = spin::GlobalMutex::new(());
    ///
    /// fn demo() {
    ///     let lock = GlobalMutex.lock();
    ///     // do something with lock
    ///     drop(lock);
    /// }
    /// ```
    pub const fn new(user_data: T) -> GlobalMutex<T>
    {
        GlobalMutex
        {
            lock: ATOMIC_BOOL_INIT,
            data: UnsafeCell::new(user_data),
        }
    }

    fn obtain_lock(&self)
    {
        while self.lock.compare_and_swap(false, true, Ordering::SeqCst) != false
        {
            // Do nothing
        }
    }

    /// Locks the spinlock and returns a guard.
    ///
    /// The returned value may be dereferenced for data access
    /// and the lock will be dropped when the guard falls out of scope.
    ///
    /// ```
    /// let mylock = spin::GlobalMutex::new(0);
    /// {
    ///     let mut data = mylock.lock();
    ///     // The lock is now locked and the data can be accessed
    ///     *data += 1;
    ///     // The lock is implicitly dropped
    /// }
    ///
    /// ```
    pub fn lock(&self) -> GlobalMutexGuard<T>
    {
        self.obtain_lock();
        GlobalMutexGuard
        {
            lock: &self.lock,
            data: unsafe { &mut *self.data.get() },
        }
    }

    /// Tries to lock the GlobalMutex. If it is already locked, it will return None. Otherwise it returns
    /// a guard within Some.
    fn try_lock(&self) -> Option<GlobalMutexGuard<T>>
    {
        if self.lock.compare_and_swap(false, true, Ordering::SeqCst) == false
        {
            Some(
                GlobalMutexGuard {
                    lock: &self.lock,
                    data: unsafe { &mut *self.data.get() },
                }
            )
        }
        else
        {
            None
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for GlobalMutex<T>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        match self.try_lock()
        {
            Some(guard) => write!(f, "GlobalMutex {{ data: {:?} }}", &*guard),
            None => write!(f, "GlobalMutex {{ <locked> }}"),
        }
    }
}

impl<T: Default> Default for GlobalMutex<T> {
    fn default() -> GlobalMutex<T> {
        GlobalMutex::new(Default::default())
    }
}

impl<'a, T> Deref for GlobalMutexGuard<'a, T>
{
    type Target = T;
    fn deref<'b>(&'b self) -> &'b T { &*self.data }
}

impl<'a, T> DerefMut for GlobalMutexGuard<'a, T>
{
    fn deref_mut<'b>(&'b mut self) -> &'b mut T { &mut *self.data }
}

impl<'a, T> Drop for GlobalMutexGuard<'a, T>
{
    /// The dropping of the GlobalMutexGuard will release the lock it was created from.
    fn drop(&mut self)
    {
        self.lock.store(false, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn try_lock() {
        let GlobalMutex = GlobalMutex::new(42);

        // First lock succeeds
        let a = GlobalMutex.try_lock();
        assert!(a.is_some());

        // Additional lock failes
        let b = GlobalMutex.try_lock();
        assert!(b.is_none());

        // After dropping lock, it succeeds again
        ::core::mem::drop(a);
        let c = GlobalMutex.try_lock();
        assert!(c.is_some());
    }
}
