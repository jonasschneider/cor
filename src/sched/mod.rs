use kbuf;

use alloc::boxed::{Box,FnBox};
use core::prelude::*;
use core::mem;
use core;
use collections::linked_list::LinkedList;



// On first access to the global, initialize it using the given expression.
// You *must* ensure that until the first access returns, no further accesses occur.
macro_rules! unsafe_lazy_static {
    ($(#[$attr:meta])* static ref $N:ident : $T:ty = $e:expr; $($t:tt)*) => {
        unsafe_lazy_static!(PRIV, $(#[$attr])* static ref $N : $T = $e; $($t)*);
    };
    ($(#[$attr:meta])* pub static ref $N:ident : $T:ty = $e:expr; $($t:tt)*) => {
        unsafe_lazy_static!(PUB, $(#[$attr])* static ref $N : $T = $e; $($t)*);
    };
    ($VIS:ident, $(#[$attr:meta])* static ref $N:ident : $T:ty = $e:expr; $($t:tt)*) => {
        unsafe_lazy_static!(MAKE TY, $VIS, $(#[$attr])*, $N);
        impl ::core::ops::Deref for $N {
            type Target = $T;
            fn deref<'a>(&'a self) -> &'a $T {
                #[inline(always)]
                fn __static_ref_initialize() -> $T { $e }

                unsafe {
                    #[inline(always)]
                    fn require_sync<T: Sync>(_: &T) { }

                    #[inline(always)]
                    unsafe fn __stability() -> &'static $T {
                        use core::cell::UnsafeCell;

                        struct SyncCell(UnsafeCell<Option<$T>>);
                        unsafe impl Sync for SyncCell {}

                        static mut DONE: bool = false;

                        static DATA: SyncCell = SyncCell(UnsafeCell::new(None));
                        if !DONE {
                          *DATA.0.get() = Some(__static_ref_initialize());
                          DONE = true;
                        }
                        match *DATA.0.get() {
                            Some(ref x) => x,
                            None => core::intrinsics::unreachable(),
                        }
                    }

                    let static_ref = __stability();
                    require_sync(static_ref);
                    static_ref
                }
            }
        }
        unsafe_lazy_static!($($t)*);
    };
    (MAKE TY, PUB, $(#[$attr:meta])*, $N:ident) => {
        #[allow(missing_copy_implementations)]
        #[allow(non_camel_case_types)]
        #[allow(dead_code)]
        $(#[$attr])*
        pub struct $N {__private_field: ()}
        #[doc(hidden)]
        pub static $N: $N = $N {__private_field: ()};
    };
    (MAKE TY, PRIV, $(#[$attr:meta])*, $N:ident) => {
        #[allow(missing_copy_implementations)]
        #[allow(non_camel_case_types)]
        #[allow(dead_code)]
        $(#[$attr])*
        struct $N {__private_field: ()}
        #[doc(hidden)]
        static $N: $N = $N {__private_field: ()};
    };
    () => ()
}


mod context;
pub mod blocking;
pub mod irq;

type Tid = usize;

#[derive(Debug)]
struct Task {
  // identity info
  id : Tid,
  desc: &'static str,

  // context switching info
  started : bool,
  stack: kbuf::Buf<'static>,
  rsp: *mut u64,
  entrypoint: Option<Entrypoint>, // will be None after launch

  // parking/scheduling info
  exited : bool,
  parked_for_irq: u16,
}

struct Entrypoint(Box<FnBox()>);

impl core::fmt::Debug for Entrypoint {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "(task entrypoint)")
    }
}

//
// STATE
//
#[derive(Debug)]
struct PerCoreState {
  runnable : LinkedList<Box<Task>>,
  current: Option<Box<Task>>,
}
// FIXME(smp): this should be per-core as well, but we have to access it from C-land, sooo...
extern {
  static mut context_switch_oldrsp_dst : u64;
  static mut context_switch_newrsp : u64;
  static mut context_switch_jumpto : u64;

  fn context_switch();
}
// Okay, this should not be a static and Rust rightly slaps us in the face for
// trying to use a mutable static thingie. However, we don't even have
// multiple cores right now, and we don't have any abstraction for per-core
// state either.
// TODO(smp): fix all of this

use sync::global_mutex::GlobalMutex;

unsafe_lazy_static! {
  static ref theState: GlobalMutex<PerCoreState> = { GlobalMutex::new(PerCoreState{runnable: LinkedList::new(), current: None}) };
}

use core::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};

// not per-cpu, but totally global
static NEXT_TASK_ID: AtomicUsize = ATOMIC_USIZE_INIT;

pub fn kyield() {
  if reschedule() {
    unsafe { context_switch(); }
  }
}


// This is the actual entry point we reach after switching tasks.
// It reads the current task from the per-CPU storage,
// and runs the actual main() function of the task.
//
// Maybe it should also do things like set up thread-local storage?
fn starttask() {
  let lebox;
  {
    let mut s = theState.lock();
    println!("Starting task {:p}", &s.current);
    let mut t = mem::replace(&mut s.current, None).unwrap();
    lebox = mem::replace(&mut t.entrypoint, None).unwrap();
    mem::replace(&mut s.current, Some(t));
  }

  FnBox::call_box(lebox.0, ());
  println!("task entrypoint returned, yielding away from us for the last time");

  {
    let mut s = theState.lock();
    let mut t = mem::replace(&mut s.current, None).unwrap();
    t.exited = true;
    mem::replace(&mut s.current, Some(t));
  }

  kyield();

  panic!("kyield() returned after exit");
}

// Defining the actual yield in Rust is unsafe because we enter the function
// on a different stack than the one that's present when leaving the function.
// This breaks one of Rust's assumptions about the machine model. So, we'll do
// it in asm!
// Returns true if a context switch should follow, false otherwise.
fn reschedule() -> bool {
  // FIXME: this is a mess

  let mut cur = theState.lock();

  println!("Task {:p} called for a reschedule.", &cur.current);
  println!("Info: {:?}", &cur.current);

  println!("LOLZERZ");

  // eep
  unsafe { context_switch_oldrsp_dst = 0 };
  unsafe { context_switch_newrsp = 0 };
  unsafe { context_switch_jumpto = 0 };


  println!("yielding, state={:?}", *cur);

  println!("again: {:?}", *cur);

  let nextval = cur.runnable.pop_front();

  println!("next: {:?}", nextval);

  let next = match nextval {
    None => {
      println!("No other task to yield to found!");
      if let Some(ref t) = cur.current {
        if t.exited {
          println!("Last task exited. Panic!");
          loop {}
        } else {
          // FIXME: this is horrible, but I want to halt somewhere.
          if t.desc.as_bytes() == "idle".as_bytes() {
            println!("Only the idle task remains. Bye!");
            panic!("Scheduler stop")
          } else {
            println!("Continuing the last task.");
            return false
          }
        }
      } else {
        println!("No current task during failing reschedule. Panic!");
        loop {}
      }
    }
    Some(mut boxt) => {
      unsafe { context_switch_newrsp = *boxt.rsp }; // pointer size..
      println!("loading sp=0x{:x}", unsafe{*boxt.rsp});

      if !boxt.started {
        boxt.started = true;
        unsafe { context_switch_jumpto = starttask as u64 };
      }
      println!("yielding to {:?}", boxt.desc);
      Some(boxt)
    }
  };

  let old = mem::replace(&mut cur.current, next);
  match old {
    Some(mut old_t) => {
      if old_t.exited {
        println!("task marked as exited, not rescheduling");
      } else {
        unsafe { context_switch_oldrsp_dst = old_t.rsp as u64; }
        cur.runnable.push_back(old_t); // TODO(perf): this allocates!!! LinkedList sucks, apparently
      }
    },
    None => {
      println!("initial switch");
    }
  }

  println!("Leaving state: {:?}", *cur);

  true
}

// This is where we enforce that only Send things can cross a task boundary.
pub fn add_task<F, T>(entrypoint: F, desc: &'static str)
  where F: FnOnce() -> T, F: Send + 'static, T: Send + 'static {
  let id = NEXT_TASK_ID.fetch_add(1, Ordering::SeqCst);

  // FIXME: Stack protection is still *totally* needed...
  let stack = kbuf::new("task stack");
  let rsp = unsafe { (stack.original_mem as u64) } +0xfff0;
  println!("Task RSP: 0x{:x}", rsp);
  let main = move || {
    entrypoint();
  };

  let t = box Task{id: id, desc: desc, entrypoint: Some(Entrypoint(box main)),
    stack: stack,
    rsp: Box::into_raw(box rsp), // FIXME: leak!
    started: false,
    exited: false,
    parked_for_irq: 0};

  theState.lock().runnable.push_back(t);
}

// Start the scheduler loop, consuming the active thread as the 'boot thread'.
pub fn exec() {
  kyield();
}


// impl core::fmt::Show for PerCoreState {
//     fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
//       write!(f, "(Sched >{}< of {})", self.current, self.runnable)
//     }
// }
