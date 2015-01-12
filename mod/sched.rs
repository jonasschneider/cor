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
  rsp: *const u8,
  rbp: *const u8,
  entrypoint: fn(),
}

// struct t *current;
struct PerCoreState {
  runnable : DList<Box<Task>>,
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
  let s = box PerCoreState{runnable: DList::new()};
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

pub fn kyield() {

}

// void kyield() {
//   // sooo, you want to yield? that's cool
//   struct t *tar = 0;

//   // we'll just grab the next task, and wrap around once we reach the end
//   if(current) {
//     tar = current->next;
//     if(!tar) tar = head;
//   } else {
//     // first run, just take the head
//     tar = head;
//   }

//   if(!tar) {
//     cor_panic("no task to yield to!");
//   }


//   if(current)
//     cor_printk("kyield: Yielding from %s to %s\n", current->desc, tar->desc);
//   else
//     cor_printk("kyield: initially launching %s\n", tar->desc);

//   // we have to be really careful with  stack allocations here, I think.
//   // actually, this should likely be asm, but whatever
//   // TODO: what else besides RSP+RBP do we need to save?
//   if(current) {
//     __asm__ volatile (
//       "mov %%rsp, %0\n"
//       "mov %%rbp, %1\n"
//       : "=r" (current->rsp), "=r" (current->rbp)
//       :
//       : "memory" // TODO: declare all the things, this is VOLATILE AS FUCK
//       );
//   }

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


