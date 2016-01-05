#include <stdint.h>
#include <stddef.h> // for size_t

typedef unsigned int uint;

typedef unsigned long ptr_t;
typedef unsigned long pid_t;

void cor_panic(const char *msg);
int cor_printk(const char *format, ...);
void putc(const char c);
void *tkalloc(size_t sz, const char *what_for, uint64_t align);

#define ALIGN(x,a)              __ALIGN_MASK(x,(__typeof__(x))(a)-1)
#define __ALIGN_MASK(x,mask)    (((x)+(mask))&~(mask))

/*
  Convert void*'s between physical and "kernel" (higher-half) memory space
*/
#define PTOK(x) ((void*)((uint64_t)(x) | 0x0000008000000000))
#define KTOP(x) ((void*)((uint64_t)(x) & (0x0000008000000000-1)))


// What follows is just helper macros

#define debug(x, ...)

/** A compile time assertion check. (http://stackoverflow.com/a/809465/950128)
 *
 *  Validate at compile time that the predicate is true without
 *  generating code. This can be used at any point in a source file
 *  where typedef is legal.
 *
 *  On success, compilation proceeds normally.
 *
 *  On failure, attempts to typedef an array type of negative size. The
 *  offending line will look like
 *      typedef assertion_failed_file_h_42[-1]
 *  where file is the content of the second parameter which should
 *  typically be related in some obvious way to the containing file
 *  name, 42 is the line number in the file on which the assertion
 *  appears, and -1 is the result of a calculation based on the
 *  predicate failing.
 *
 *  \param predicate The predicate to test. It must evaluate to
 *  something that can be coerced to a normal C boolean.
 *
 *  \param file A sequence of legal identifier characters that should
 *  uniquely identify the source file in which this condition appears.
 */
#define CASSERT(predicate) _impl_CASSERT_LINE(predicate,__LINE__,__FILE__)

#define _impl_PASTE(a,b) a##b
#define _impl_CASSERT_LINE(predicate, line, file) \
    typedef char _impl_PASTE(assertion_failed_##file##_,line)[2*!!(predicate)-1];
