use myheap;
use kbuf;

use boxed::Box;
use core::prelude::*;
use core::mem;
use core;
use mydlist::DList;

type Tid = u64;

struct Task {
  id : Tid,
  desc: &'static str,

  ran : bool,
  stack: kbuf::Buf<'static>,
  rsp: u64,
  rbp: u64,
  entrypoint: fn(),
}

// struct t *current;
struct PerCoreState {
  runnable : DList<Box<Task>>,
  current: Option<Box<Task>>,
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
  let rsp = (stack.original_mem as u64) +0xff0;
  let t = box Task{id: id, desc: desc, entrypoint: entrypoint,
    stack: stack,
    rsp: rsp,
    rbp: rsp,
    ran: false};

  // unsafe because we have to access the global state.. ugh
  unsafe { (*theState).runnable.push_back(t); }
}

// Start the scheduler loop, consuming the active thread as the 'boot thread'.
pub fn exec() {
  kyield();
}

fn starttask() {
  unsafe {
    let ref c = (*theState).current;  // there should always be a current after context switch

    match c {
      &None => {
        println!("no task after afterswitch?!");
      }
      &Some(ref t) => {
        println!("launching task entrypoint");
        (t.entrypoint)();
        println!("ohmygod entrypoint returned, cleanup here pls alert alert");
        while(true) {}
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
        println!("loading {} from ", newRSP as *mut u64);

        if(boxt.ran == false) {
          boxt.ran = true;
          afterswitch = starttask as u64;
        }
        println!("yielding to {}", boxt.desc);
        Some(boxt)
      }
    };


    let old = mem::replace(&mut s.current, next);
    match old {
      Some(mut old_t) => {
        oldRSP_dst = mem::transmute(&old_t.rsp);
        oldRBP_dst = mem::transmute(&old_t.rsp);
        println!("old value {:x} is stored at {}", old_t.rsp, oldRSP_dst);
        s.runnable.push_back(old_t); // TODO(perf): this allocates!!! DList sucks, apparently
      },
      None => {
        println!("initial switch");
      }
    };


    // sooo, this is a bit hairy. Original C:
    // if(current) {
    //   __asm__ volatile (
    //     "mov %%rsp, %0\n"
    //     "mov %%rbp, %1\n"
    //     : "=r" (current->rsp), "=r" (current->rbp)
    //     :
    //     : "memory" // TODO: declare all the things, this is VOLATILE AS FUCK
    //     );
    // }
    // // aaand switcharoo
    // current = tar;
    // __asm__ volatile (
    //   "mov %0, %%rsp\n"
    //   "mov %1, %%rbp\n"
    //   :
    //   : "r" (current->rsp), "r"(current->rbp)
    //   :
    //   );
    // if(current->ran == 0) {
    //   current->ran = 1;
    //   current->entry();
    //   // Right here, I think, is where we have to worry about a thread exiting
    //   // We can't just return since that will blow the stack, I think
    //   cor_panic("A thread exited, I don't know what to do");
    // }
    //
    // We want to do all of this as safely as possible. I think, 'safely' in this
    // context actually means in a single block of asm. It may not use the stack, and it may not
    // do function calls (since we mess with the stack).
    // We have collected all the data we need from above. Rust should have all of these
    // local variables on the stack, so we'll just put them all in registers,
    // then switch out the stack, and then restore as appropriate.
    // TODO: think if the exact ourobouros-recursion-property we're enforcing here is just reentrancy.
    //       Reentrancy is certainly a part of it.
    // 'Appropriate' is:
    //   (a) For new tasks, launch starttask(), which will launch the task's entrypoint and handles when it exits
    //   (b) For existing tasks, just return, like a call to kyield() should

    asm!( // FIXME: the common case should not jump
      // save old things
      "mov %rsp, ($0)
       mov %rbp, ($1)
       mov $2, %rsp
       mov $3, %rbp
       cmp $$0, $4
       jne go_to_starttask
       jmp just_continue
      go_to_starttask:
       jmp *$4
      just_continue:

       # FIXME: THIS IS A HUGE HACK AND SUPER UNSAFE.
       # -CUE WARNING LIGHTS-
       # This is actually worse than the context switch itself.
       # We don't let Rust finish up with its own stack frame,
       # skipping Drop() handlers inserted by LLVM because they depend on the old stack.
       # This is horrible and it seems like we should do context-switching outside of Rustland
       # entirely.
       pop %rbp
       retq
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
