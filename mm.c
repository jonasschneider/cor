#include "common.h"

#pragma pack(push, 1)
struct memmap_entry {
  uint64_t base;
  uint64_t length;
  uint32_t type;
  uint32_t flags;
};
#pragma pack(pop)

struct region {
  size_t limit;
  void *base;
  size_t used;
};
struct region source_region;

void mm_init() {
  unsigned int i = 0;
  size_t largest_limit = 0;
  void *largest_base = 0;

  while(1) {
    if(i > 32) { // TODO: figure out this limit
      cor_panic("too many memory map entries");
    }
    void *p = (void*)0x8000+(i++*32);
    struct memmap_entry *e = (struct memmap_entry *)p;
    if(e->base == 0xdeadbeef) break;
    cor_printk("memory region %u: base=%lx, siz=%lx, t=%x, flags=%x\n",
      i, e->base, e->length, e->type, e->flags);
    if(e->length > largest_limit) {
      largest_base = (void*)e->base;
      largest_limit = e->length;
    }
  }

  // pretty much the best implementation ever: just ignore all segments but the biggest
  if(largest_base == 0 || largest_limit < 0x10000) {
    cor_panic("did not find a valid memory region");
  }
  source_region.base = largest_base;
  source_region.limit = largest_limit;
  source_region.used = 0;

  cor_printk(" kalloc region starts at %p, limit %x. ", source_region.base, source_region.limit);
}


void *tkalloc(size_t s, const char *what_for, uint64_t align) {
  if(source_region.used + s > source_region.limit) {
    cor_printk("OOM: requested %x, but already used %x out of %x\n",s,source_region.used,source_region.limit);
    cor_panic("");
    return 0; // fix warning
  }
  source_region.used = ALIGN(source_region.used, align);
  void *p = source_region.base + source_region.used;
  source_region.used += s;

  cor_printk("- tkalloc: %s (%x) --> %p\n", what_for, s, p);

  return p;
}
