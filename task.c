#include "common.h"
#include "task.h"

struct task_table_entry *the_task; // yes, we can only have 1 right now

void *initial_pagetable = (void*)0x1000;

struct task_table_entry *task_new() {
  the_task = (struct task_table_entry *)tkalloc(sizeof(struct task_table_entry), "new task struct", 1);
  the_task->brk = 0;

  // fill page table
  the_task->page_table_base = tkalloc(0x5000, "task page table", 0x1000);
  for(size_t j = 0; j < 0x5000; j++) { // TODO: memcpy
    char *loadsrc = initial_pagetable + j;
    char *loadtarget = the_task->page_table_base + j;
    *loadtarget = *loadsrc;
  }

  *(ptr_t*)(the_task->page_table_base+0x0000) = ((ptr_t)the_task->page_table_base+0x1003)|4;
  *(ptr_t*)(the_task->page_table_base+0x1000) = ((ptr_t)the_task->page_table_base+0x2003)|4;

  // memzero out the inner page table
  for(size_t j = 0x3000; j < 0x4000; j++) { // TODO: memzero
    char *t = the_task->page_table_base + j;
    *t = 0;
  }

  return the_task;
}

int task_addpage(struct task_table_entry *t, void *offset) {
  void *phys = tkalloc(0x1000, "task page", 0x1000);
  if(t->page_table_useddir != 0) {
    if(t->page_table_useddir != (void*)((ptr_t)offset&0xffffff)) {
      cor_printk("Attempted to add pageoffset %p to task table entry with dir=%p", offset, t->page_table_useddir);
      return -1;
    }
    cor_panic("lolbranch");
    return -1;
  }

  uint16_t whichpage = (ptr_t)offset >> 12 & ((1<<9)-1);
  uint16_t whichtbl = (ptr_t)offset >> 21 & ((1<<9)-1);
  cor_printk("Mapping page %x in table %x\n", whichpage, whichtbl);

  uint64_t *pdloc = (uint64_t *)((ptr_t)t->page_table_base + 0x2000 + whichtbl*8);
  uint64_t pdentry = ((ptr_t)t->page_table_base + 0x3000) | 3 | 4;
  cor_printk("Page directory entry %p --> %p\n", pdloc, pdentry);
  *pdloc = pdentry;

  uint64_t *ptloc = (uint64_t *)((ptr_t)t->page_table_base + 0x3000 + whichpage*8);
  uint64_t ptentry = (ptr_t)phys | 3 | 4;
  cor_printk("Page table entry %p --> %p\n", ptloc, ptentry);
  *ptloc = ptentry;

  return 0;
}

void task_enter_memspace(struct task_table_entry *t) {
  cor_printk("Entering task memspace\n");
  __asm__ (
    "mov %0, %%rax\n"
    "mov %%rax, %%cr3"
    :
    : "r"(t->page_table_base)
    : "rax"
    );
}

// TODO fix this
#define MAXALLOC 0x10000
uint64_t syscall_moremem(uint64_t insize) {
  cor_printk("syscall_moremem() size=%x\n", insize);

  size_t size = (size_t)insize;
  if(0 < size && size <= MAXALLOC) {
    cor_printk("addpage'ing at brk %p\n", the_task->brk);
    task_addpage(the_task, the_task->brk);
    void *oldbrk = the_task->brk;
    the_task->brk += 0x1000;
    return (uint64_t)oldbrk;
  } else {
    cor_printk("invalid size given for syscall_moremem()\n");
    return -1;
  }
}
