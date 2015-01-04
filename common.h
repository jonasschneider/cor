#include <stdint.h>

typedef unsigned int uint;

typedef unsigned long ptr_t;

void cor_panic(const char *msg);
int cor_printk(const char *format, ...);
void putc(const char c);
void *tkalloc(size_t sz, const char *what_for, uint64_t align);

#define ALIGN(x,a)              __ALIGN_MASK(x,(__typeof__(x))(a)-1)
#define __ALIGN_MASK(x,mask)    (((x)+(mask))&~(mask))
