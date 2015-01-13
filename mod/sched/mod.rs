use myheap;
use kbuf;

use boxed::Box;
use core::prelude::*;
use core::mem;
use core;
use mydlist::DList;

mod context;

type Tid = u64;

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

impl core::fmt::Show for Task {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
      let s = if self.exited { "dead" } else { "alive" };
      write!(f, "[{} - {} - {}]", self.id, self.desc, s)
    }
}

// struct t *current;
struct PerCoreState {
  runnable : DList<Box<Task>>,
  current: Option<Box<Task>>,
}


impl core::fmt::Show for PerCoreState {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
      write!(f, "(Sched. >{}< of {})", self.current, self.runnable)
    }
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

fn starttask() {
  unsafe {
    let ref mut c = (*theState).current;  // there should always be a current after context switch
    match c {
      &None => {
        println!("no task after afterswitch?!");
      }
      &Some(ref mut t) => {
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

pub fn kyield() {
  // unsafe because we have to access the global state..
  // we're doing lots more of unsafe operations in here.
  // TODO: finer-grained unsafe blocks here, and think harder about everything unsafe
  unsafe {
    let ref mut s = *theState;
    println!("yielding with state={}", s);

    // eep
    let mut oldRSP_dst = 0 as *mut u64;
    let mut oldRBP_dst = 0 as *mut u64;
    let mut newRSP = 0;
    let mut newRBP = 0;

    // this is the best thing ever: in case we're entering anew, we put the
    // address of starttask() in here
    let mut afterswitch = 0;


    let next = match s.runnable.pop_front() {
      None => {
        println!("nothing to yield to! panic!");
        while(true) {}
        core::intrinsics::unreachable();
        // FIXME: http://doc.rust-lang.org/std/macro.panic!.html
        //panic!();
      }
      Some(mut boxt) => {
        newRSP = boxt.rsp as u64; // POINTER SIZES FTW
        newRBP = boxt.rbp as u64;
        println!("loading sp=0x{:x}, bp=0x{:x}", newRSP, newRBP);

        if(boxt.started == false) {
          boxt.started = true;
          afterswitch = starttask as u64;
        }
        println!("yielding to {}", boxt.desc);
        Some(boxt)
      }
    };


    let old = mem::replace(&mut s.current, next);
    match old {
      Some(mut old_t) => {
        if old_t.exited {
          println!("task marked as exited, not rescheduling");
        } else {
          oldRSP_dst = mem::transmute(&old_t.rsp);
          oldRBP_dst = mem::transmute(&old_t.rbp);
          println!("old RSP is stored at {}", oldRSP_dst);
          s.runnable.push_back(old_t); // TODO(perf): this allocates!!! DList sucks, apparently
        }
      },
      None => {
        println!("initial switch");
      }
    };


    // sooo, the context switch itself is a bit hairy. Our goals are:
    //
    //  1. Save the old task's %rsp and %rbp into its task struct so we know
    //     how to resume
    //  2. Load the new task's %rsp and %rbp from its task struct
    //  3. And then comes something interesting:
    //    - If the new task has never been launched yet: "Call" its entrypoint
    //      -- I say "call" because since we're not in a valid stack frame,
    //      this call can never return without breaking stuff. So, we wrap it
    //      in a call around starttask().
    //    - Otherwise, the new stack frame puts us into a call to kyield(),
    //      the frame that called us is code from within the resumed task. So
    //      we should just return like
    //      a reasonable citizen.

    // We want to do all of this as safely as possible. I think, 'safely' in this
    // context actually means in a single block of asm. It may not use the stack, and it may not
    // do function calls (since we mess with the stack).
    // We have collected all the data we need from above. We need to put it all in registers
    // so we don't depend on the stack at all anymore.
    // TODO: think if the exact ourobouros-recursion-property we're enforcing here is just reentrancy.
    //       Reentrancy is certainly a part of it.
    asm!(
      "cmp $$0, $0
       je kyield_drop_old
       mov %rsp, ($0)
       mov %rbp, ($1)
      kyield_drop_old:
       mov $2, %rsp
       mov $3, %rbp
       cmp $$0, $4
       jne kyield_go_to_starttask
       jmp kyield_just_continue # FIXME(perf): the common case should not have to branch
      kyield_go_to_starttask:
       jmp *$4
      kyield_just_continue:

       # FIXME: THIS IS A HUGE HACK AND SUPER UNSAFE.
       # -CUE WARNING LIGHTS-
       # This is actually worse than the context switch itself.
       # We don't let Rust finish up with its own stack frame,
       # skipping Drop() handlers inserted by LLVM because they depend on the old stack.
       # This is horrible and it seems like we should do context-switching outside of Rustland
       # entirely.
       # I still have this idea about modding the return address of this stack frame.
       # It would point to do_context_switch, which is itself defined in a .s somewhere
       # and receives its arguments (i.e. the stuff we give it here) via a static (actually, cpu-local)
       # memory location somewhere. (it can't be the registers and it can't be the stack...)
       leave
       ret
       "
      : // no writes
      : "r"(oldRSP_dst),
        "r"(oldRBP_dst),
        "r"(newRSP)
        "r"(newRBP)
        "r"(afterswitch)
      : "rax", "rsp", "rbp"
      : // options here
      );
  }
}

pub fn init() {
  println!("initing sched!");
  let s = box PerCoreState{runnable: DList::new(), current: None};
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
