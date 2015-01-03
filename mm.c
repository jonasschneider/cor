#include "printk.h"
#include "common.h"

#pragma pack(push, 1)
struct memmap_entry {
  uint64_t base;
  uint64_t length;
  uint32_t type;
  uint32_t flags;
};
#pragma pack(pop)

void mm_init() {
  unsigned int i = 0;
  while(1) {
    if(i > 32) { // TODO: figure out this limit
      cor_panic("too many memory map entries");
    }
    void *p = (void*)0x8000+(i++*32);
    struct memmap_entry *e = (struct memmap_entry *)p;
    if(e->base == 0xdeadbeef) break;
    cor_printk("memory segment %u: base=%lx, siz=%lx, t=%x, flags=%x\n",
      i, e->base, e->length, e->type, e->flags);
  }
}
