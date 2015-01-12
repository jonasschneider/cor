
use myheap;

use boxed::Box;
use core::prelude::*;
use core::mem;
use core;

// #include "common.h"

// struct t {
//   void *rsp;
//   void (*entry)();
//   void *rbp;
//   struct t *next;
//   const char *desc;
//   int ran;
//   pid_t pid;
// };
// struct t *head = 0;
// struct t *current;
// pid_t next_pid = 0;

type Tid = u64;

struct Task {
  id : Tid,
}

enum TaskListNode {
  Entry(Box<Task>, Box<TaskListNode>),
  End
}

impl core::fmt::Show for TaskListNode {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        // The `f` value implements the `Writer` trait, which is what the
        // write! macro is expecting. Note that this formatting ignores the
        // various flags provided to format strings.
        match self {
          &TaskListNode::Entry(ref t, ref n)  => write!(f, "(n: present), {}", n),
          &TaskListNode::End  => write!(f, "(end)")
        }

    }
}

struct PerCoreState {
  runnable_list_head : Box<TaskListNode>,
}

// Okay, this should not be a static and Rust rightly slaps us in the face for
// trying to use a mutable static thingie. However, we don't even have
// multiple cores right now, and we don't have any abstraction for per-core
// state either.
// TODO(smp): fix all of this
static mut theState : *mut PerCoreState = 0 as *mut PerCoreState;

pub fn init() {
  println!("initing sched!");
  let s = box PerCoreState{runnable_list_head: box TaskListNode::End};
  unsafe  {
    // This will (per IRC) consume the box, and turn it into a pointer
    // to the thing that was in the box (the box itself isn't a struct anywhere in memory)
    // This also means that the box won't be dropped once we leave this function,
    // but will instead 'leak' -- which is exactly what we want.
    theState = mem::transmute(s);
  }
  unsafe { println!("sched is so up: {}", (*theState).runnable_list_head); }
}


//
// void kyield();

// void sched_init() {

// }

// pid_t sched_add(void (*entry)(), const char *desc) {
//   // fill in the identity parts
//   struct t *t1 = (struct t*)tkalloc(sizeof(struct t), "struct t", 0x10);
//   t1->pid = next_pid++;
//   t1->desc = desc;

//   // Set up the scheduling info
//   t1->rsp = t1->rbp = tkalloc(0x1000, "task stack", 0x10) + 0x0ff0; // 4k by default
//   t1->entry = entry;
//   t1->ran = 0;

//   // Insert it into the list
//   t1->next = head;
//   head = t1;

//   return t1->pid;
// }

// void sched_exec() {
//   // This one's actually surprisingly easy
//   kyield();
// }

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


