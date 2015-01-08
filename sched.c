#include "common.h"

struct t {
  void *rsp;
  void *rip;
  struct t *next;
  const char *desc;
}
struct t *head;
struct t *current;

void sched_init() {
  t1 = (struct t*)tkalloc(sizeof(struct t), "struct t for t1", 0x10);
  t1->rsp = tkalloc(0x1000, "stack for t1", 0x10);
  t1->rip = (void*)thread1;
  t1->desc = "t1";

  t2 = (struct t*)tkalloc(sizeof(struct t), "struct t for t2", 0x10);
  t2->rsp = tkalloc(0x1000, "stack for t2", 0x10);
  t2->rip = (void*)thread2;
  t2->desc = "t2";

  head = t1;
  t1->next = t2;
  t2->next = 0;

  // GOOO
  current = t1;
  kyield();
}

void kyield() {
  // sooo, you want to yield? that's cool
  struct t *cur = head;
  struct t *next = 0;
  while(cur) {
    if(cur == current) {
      // don't yield to yourself
      continue
    }

    // insert priorities here
    next = cur;
  }

  if(!next) {
    cor_panic("no task to yield to!");
  }

  cor_printk("kyield: Yielding from %s to %s\n", current->desc, next->desc);

  // no stack allocations allowed after here! this should actually be asm, i guess, but whatever
  // TODO: what else besides RSP do we need to save?

  __asm__ volatile (
    "mov %%rsp, %0\n"
    : "=r" (current->rsp)
    :
    : "memory" // TODO: declare all the things, this is VOLATILE AS FUCK
    );
  current->rip = done; // when reentering, don't switch things, but jump to the end of kyield (because recursion)

  __asm__ volatile (
    "mov %0, %%rsp\n"
    "jmp %1"
    :
    : "r" (next->rsp), "r" (next->rip)
    :
    );

done:
}

void thread1() {
  while(1) {
    cor_printk("Hello from t1!\n");
    for(int i=0;i<100000;i++); // we can't really sleep yet
    cor_printk("t1 yielding now.\n");
    kyield();
    cor_printk("kyield'ed to t1\n");
  }
}

void thread2() {
  while(1) {
    cor_printk("Hello from t1!\n");
    for(int i=0;i<100000;i++); // we can't really sleep yet
    cor_printk("t1 yielding now.\n");
    kyield();
    cor_printk("kyield'ed to t1\n");
  }
}
