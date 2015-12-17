use kbuf;

use alloc::boxed::Box;
use core::prelude::*;
use core::mem;
use core;
use collections::linked_list::LinkedList;

mod context;

type Tid = u64;

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
  entrypoint: fn(),

  // parking/scheduling info
  exited : bool,
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
}
// Okay, this should not be a static and Rust rightly slaps us in the face for
// trying to use a mutable static thingie. However, we don't even have
// multiple cores right now, and we don't have any abstraction for per-core
// state either.
// TODO(smp): fix all of this
static mut theState : *mut PerCoreState = 0 as *mut PerCoreState;

// Interestingly, this shouldn't be CPU-local, but still global
// Use an ArcRW or something?
static mut nextTid : Tid = 0;
fn makeNextTid() -> u64 {
  // unsafe because global & nonatomic
  unsafe {
    nextTid += 1;
    return nextTid
  }
}
//
// END STATE
//

pub fn kyield() {
  reschedule();
  unsafe { context_switch(); }
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
        (t.entrypoint)();
        println!("task entrypoint returned, yielding away from us for the last time");
        t.exited = true;
        kyield();
        println!("PANIC! kyield() returned after exit");
        while(true) { }
      }
    }

  }
}


// Defining the actual yield in Rust is unsafe because we enter the function
// on a different stack than the one that's present when leaving the function.
// This breaks one of Rust's assumptions about the machine model. So, we'll do
// it in asm!
fn reschedule() {
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
        println!("nothing to yield to! panic!");
        while(true) {}
        core::intrinsics::unreachable();
        // FIXME: http://doc.rust-lang.org/std/macro.panic!.html
        //panic!();
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

  // we don't do the actual context switch here, see yield
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

pub fn add_task(entrypoint : fn(), desc : &'static str) {
  let id = makeNextTid();

  let stack = kbuf::new("task stack");
  let rsp = unsafe { (stack.original_mem as u64) } +0x3ff0;
  println!("Task RSP: 0x{:x}", rsp);
  let t = box Task{id: id, desc: desc, entrypoint: entrypoint,
    stack: stack,
    rsp: rsp,
    rbp: rsp,
    started: false,
    exited: false};

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
