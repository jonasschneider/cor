use kbuf;

use alloc::boxed::{Box,FnBox};
use core::prelude::*;
use core::mem;
use core;
use collections::linked_list::LinkedList;

mod context;
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
  rsp: u64,
  rbp: u64,
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
  static mut context_switch_oldrbp_dst : u64;
  static mut context_switch_newrsp : u64;
  static mut context_switch_newrbp : u64;
  static mut context_switch_jumpto : u64;

  fn context_switch();

  static mut irq_log : [u8; 256];
}
// Okay, this should not be a static and Rust rightly slaps us in the face for
// trying to use a mutable static thingie. However, we don't even have
// multiple cores right now, and we don't have any abstraction for per-core
// state either.
// TODO(smp): fix all of this
static mut theState : *mut PerCoreState = 0 as *mut PerCoreState;

use core::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};

// not per-cpu, but totally global
static NEXT_TASK_ID: AtomicUsize = ATOMIC_USIZE_INIT;

pub fn kyield() {
  if reschedule() {
    unsafe { context_switch(); }
  }
}

pub fn reset_irq(irq: u16) {
  unsafe { irq_log[irq as usize] = 0; }
}

pub fn park_until_irq(irq: u16) {
  unsafe {
    let ref mut c = (*theState).current;  // there should always be a current after context switch
    match c {
      &mut None => {
        println!("no task after afterswitch?!");
      }
      &mut Some(ref mut t) => {
        println!("sched: Parking {:?} until irq {}", t, irq);
        t.parked_for_irq = irq
      }
    }
  }
  while unsafe { irq_log[irq as usize] == 0 } {
    println!("Irq {} still hasn't been logged, continuing park..", irq);
    kyield();
  }
  println!("Irq {} logged, exiting parking!", irq);
  reset_irq(irq); // FIXME: need better global handling of this
}


// This is the actual entry point we reach after switching tasks.
// It reads the current task from the per-CPU storage,
// and runs the actual main() function of the task.
//
// Maybe it should also do things like set up thread-local storage?
fn starttask() {
  unsafe {
    let ref mut c = (*theState).current;  // there should always be a current after context switch
    match c {
      &mut None => {
        println!("no task after afterswitch?!");
      }
      &mut Some(ref mut t) => {
        println!("launching task entrypoint");
        let entryp = mem::replace(&mut t.entrypoint, None).unwrap();
        let lebox = entryp.0;
        FnBox::call_box(lebox, ());
        println!("task entrypoint returned, yielding away from us for the last time");
        t.exited = true;
        kyield();
        panic!("kyield() returned after exit");
      }
    }

  }
}

// Defining the actual yield in Rust is unsafe because we enter the function
// on a different stack than the one that's present when leaving the function.
// This breaks one of Rust's assumptions about the machine model. So, we'll do
// it in asm!
// Returns true if a context switch should follow, false otherwise.
fn reschedule() -> bool {
  // unsafe because we have to access the global state..
  // we're doing lots more of unsafe operations in here.
  // TODO: finer-grained unsafe blocks here, and think harder about everything unsafe
  unsafe {
    // eep
    context_switch_oldrsp_dst = 0;
    context_switch_oldrbp_dst = 0;
    context_switch_newrsp = 0;
    context_switch_newrbp = 0;
    context_switch_jumpto = 0;

    let ref mut s = *theState;
    println!("yielding with state={:?}, data={:?}", s, context_switch_oldrbp_dst);

    let next = match s.runnable.pop_front() {
      None => {
        println!("No other task to yield to found!");
        if let Some(ref t) = s.current {
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
        context_switch_newrsp = boxt.rsp as u64; // POINTER SIZES FTW
        context_switch_newrbp = boxt.rbp as u64;
        println!("loading sp=0x{:x}, bp=0x{:x}", context_switch_newrsp, context_switch_newrbp);

        if(boxt.started == false) {
          boxt.started = true;
          context_switch_jumpto = starttask as u64;
        }
        println!("yielding to {:?}", boxt.desc);
        Some(boxt)
      }
    };

    let old = mem::replace(&mut s.current, next);
    match old {
      Some(mut old_t) => {
        if old_t.exited {
          println!("task marked as exited, not rescheduling");
        } else {
          context_switch_oldrsp_dst = mem::transmute(&old_t.rsp);
          context_switch_oldrbp_dst = mem::transmute(&old_t.rbp);
          s.runnable.push_back(old_t); // TODO(perf): this allocates!!! LinkedList sucks, apparently
        }
      },
      None => {
        println!("initial switch");
      }
    }
  }

  true
}

pub fn init() {
  println!("initing sched!");
  let s = box PerCoreState{runnable: LinkedList::new(), current: None};
  unsafe  {
    // This will (per IRC) consume the box, and turn it into a pointer
    // to the thing that was in the box (the box itself isn't a struct anywhere in memory)
    // This also means that the box won't be dropped once we leave this function,
    // but will instead 'leak' -- which is exactly what we want.
    theState = mem::transmute(s);
  }
}

// This is where we enforce that only Send things can cross a task boundary.
pub fn add_task<F, T>(entrypoint: F, desc: &'static str)
  where F: FnOnce() -> T, F: Send + 'static, T: Send + 'static {
  let id = NEXT_TASK_ID.fetch_add(1, Ordering::SeqCst);

  let stack = kbuf::new("task stack");
  let rsp = unsafe { (stack.original_mem as u64) } +0x3ff0;
  println!("Task RSP: 0x{:x}", rsp);
  let main = move || {
    entrypoint();
  };

  let t = box Task{id: id, desc: desc, entrypoint: Some(Entrypoint(box main)),
    stack: stack,
    rsp: rsp,
    rbp: rsp,
    started: false,
    exited: false,
    parked_for_irq: 0};

  // unsafe because we have to access the global state.. ugh
  unsafe { (*theState).runnable.push_back(t); }
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
