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
  rsp: *mut u8,
  rbp: *mut u8,
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

pub unsafe fn init() {
  println!("initing sched!");
  let s = box PerCoreState{runnable: DList::new(), current: None};
  unsafe  {
    // This will (per IRC) consume the box, and turn it into a pointer
    // to the thing that was in the box (the box itself isn't a struct anywhere in memory)
    // This also means that the box won't be dropped once we leave this function,
    // but will instead 'leak' -- which is exactly what we want.
    theState = mem::transmute(s);
  }
  //unsafe { println!("sched is so up: {}", (*theState).runnable_list_head); }
}


pub fn add_task(entrypoint : fn(), desc : &'static str) {
  let id = makeNextTid();

  let stack = kbuf::new("task stack");
  let t = box Task{id: id, desc: desc, entrypoint: entrypoint,
    stack: stack,
    rsp: stack.original_mem,
    rbp: stack.original_mem,
    ran: false};

  // unsafe because we have to access the global state.. ugh
  unsafe { (*theState).runnable.push_front(t); }
}

pub fn exec() {
  kyield();
}

fn afterswitch() {
  unsafe {
    let ref c = (*theState).current;  // there should always be a current after context switch

    match c {
      &None => {
        println!("no task after afterswitch?!");
      }
      &Some(ref t) => {
        if t.ran == false {
          println!("launching task entrypoint");
          (t.entrypoint)();
        } else {
          println!("just re-entering");
        }
      }
    }

  }
}

extern "C" {
  fn sched_contextswitch(
    oldRSP_dst : *mut u64,
    oldRBP_dst : *mut u64,

    newRSP : u64,
    newRBP : u64,

    afterswitch : fn(),
  );
}

pub fn kyield() {
  // unsafe because we have to access the global state..
  unsafe {
    let ref mut s = *theState;

    let mut next : Option<Box<Task>> = None;
    // eep
    let mut oldRSP_dst = 0 as *mut u64;
    let mut oldRBP_dst = 0 as *mut u64;
    let mut newRSP = 0;
    let mut newRBP = 0;


    match s.runnable.pop_front() {
      None => {
        println!("nothing to yield to! panic!");
        // FIXME: http://doc.rust-lang.org/std/macro.panic!.html
        //panic!();
      }
      Some(boxt) => {
        newRSP = boxt.rsp as u64; // POINTER SIZES FTW
        newRBP = boxt.rsp as u64;
        next = Some(boxt);
      }
    }


    let old = mem::replace(&mut s.current, next);
    match old {
      Some(old_t) => {
        println!("switching away");
        s.runnable.push_back(old_t);
      },
      None => {
        println!("initial switch");
      }
    };


    sched_contextswitch(
      oldRSP_dst,
      oldRBP_dst,

      newRSP,
      newRBP,

      afterswitch,
    );
  }
}


//   // aaand switcharoo
//   current = tar;

//   __asm__ volatile (
//     "mov %0, %%rsp\n"
//     "mov %1, %%rbp\n"
//     :
//     : "r" (current->rsp), "r"(current->rbp)
//     :
//     );

//   if(current->ran == 0) {
//     current->ran = 1;
//     current->entry();
//     // Right here, I think, is where we have to worry about a thread exiting
//     // We can't just return since that will blow the stack, I think
//     cor_panic("A thread exited, I don't know what to do");
//   }

// }


