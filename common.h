typedef unsigned int uint;

typedef unsigned char uint8_t;
typedef unsigned short uint16_t;
typedef unsigned int uint32_t;
typedef unsigned long uint64_t;

typedef unsigned long ptr_t;

void cor_panic(const char *msg);
int cor_printk(const char *format, ...);
void *tkalloc(size_t sz, const char *what_for, uint64_t align);

#define ALIGN(x,a)              __ALIGN_MASK(x,(__typeof__(x))(a)-1)
#define __ALIGN_MASK(x,mask)    (((x)+(mask))&~(mask))
