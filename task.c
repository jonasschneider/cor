#include "common.h"
#include "task.h"

struct task_table_entry *the_task; // yes, we can only have 1 right now

struct task_table_entry *task_new() {
  the_task = (struct task_table_entry *)tkalloc(sizeof(struct task_table_entry), "new task struct");
  the_task->page_table_base = tkalloc(0x4000, "task page table");
  the_task->brk = 0;
  the_task->first_section = 0;
  return the_task;
}

struct task_section *task_add_section(struct task_table_entry *t, char type, size_t size) {
  struct task_section *s = (struct task_section *)tkalloc(sizeof(struct task_section), "task section desciptor");
  s->type = type;
  s->size = ALIGN(size, 0x1000);
  cor_printk("aligned size is %x\n", s->size);
  s->base = tkalloc(s->size, "task section data");

  // insert into list
  s->next = t->first_section;
  t->first_section = s;
  return s;
}

// TODO fix this
#define MAXALLOC 0x10000
uint64_t syscall_moremem(uint64_t insize) {
  cor_printk("syscall_moremem() size=%x\n", insize);

  size_t size = (size_t)insize;
  if(0 < size && size <= MAXALLOC) {
    // do it
    return 0;
  } else {
    cor_printk("invalid size given for syscall_moremem()\n");
    return -1;
  }
}
