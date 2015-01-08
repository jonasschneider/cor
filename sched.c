#include "common.h"

struct t {
  void *rsp;
  void *rip;
  void *rbp;
  struct t *next;
  const char *desc;
};
struct t *head;
struct t *current;

void kyield();
void thread1();
void thread2();

void sched_init() {
  struct t *t1 = (struct t*)tkalloc(sizeof(struct t), "struct t for t1", 0x1000);
  t1->rsp = t1->rbp = tkalloc(0x1000, "stack for t1", 0x1000);
  t1->rip = (void*)thread1;
  t1->desc = "t1";

  struct t *t2 = (struct t*)tkalloc(sizeof(struct t), "struct t for t2", 0x1000);
  t2->rsp = t2->rbp = tkalloc(0x1000, "stack for t2", 0x1000);
  t2->rip = (void*)thread2;
  t2->desc = "t2";

  head = t1;
  t1->next = t2;
  t2->next = 0;

  // GOOO
  current = t1;
  kyield();
  thread1();
}

void kyield() {
  // sooo, you want to yield? that's cool
  struct t *cur = head;
  struct t *tar = 0;

  // we'll just grab any task that's not us
  while(cur) {
    cor_printk("checking %s\n", cur->desc);
    if(cur != current) {
      // then it's a candidate.
      // insert priorities here
      tar = cur;
    }
    cur = cur->next;
  }

  if(!tar) {
    cor_panic("no task to yield to!");
  }

  cor_printk("kyield: Yielding from %s to %s\n", current->desc, tar->desc);

  // no stack allocations allowed after here! this should actually be asm, i guess, but whatever
  // TODO: what else besides RSP do we need to save?

  __asm__ volatile (
    "mov %%rsp, %0\n"
    "mov %%rbp, %1\n"
    : "=r" (current->rsp), "=r" (current->rbp)
    :
    : "memory" // TODO: declare all the things, this is VOLATILE AS FUCK
    );

  // Please write multiple paragraphs about this and how it's a horrible thing and works on gcc only
  void *le_rip = &&done;
  current->rip = le_rip; // when reentering, don't switch things, but jump to the end of kyield (because recursion)


  // aaand switcharoo
  current = tar;

  __asm__ volatile (
    "mov %1, %%rsp\n"
    "mov %2, %%rbp\n"
    "jmp %0"
    :
    : "r" (current->rip), "r" (current->rsp), "r"(current->rbp)
    :
    );

done:
  // we need at least one instruction here...
  __asm__ volatile("nop");
}

void thread1() {
  while(1) {
    cor_printk("Hello from t1! working..\n");
    for(int i=0;i<100000000;i++); // we can't really sleep yet
    cor_printk("t1 yielding now.\n");
    kyield();
    cor_printk("we kyield'ed to t1\n");
  }
}

void thread2() {
  while(1) {
    cor_printk("Hello from t2! working..\n");
    for(int i=0;i<100000000;i++); // we can't really sleep yet
    cor_printk("t2 yielding now.\n");
    kyield();
    cor_printk("we kyield'ed to t2\n");
  }
}
