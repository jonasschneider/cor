#include "../common.h"

size_t rust_allocd = 0;

// TODO: rust actually has these as uint, not size_t... bad?
void *rust_allocate(size_t size, size_t align) {
  rust_allocd += size;
  void *ptr = tkalloc(size, "rust_allocate()", align);
  __builtin_memset(ptr, 0, size);
  return ptr;
}

// TODO: review these.
void memset(void *dst, int fill, size_t n) {
  size_t i = n;
  while(i-- > 0) {
    *((uint8_t*)dst++) = fill;
  }
}
void memcpy(void *dst, void *src, size_t n) {
  size_t i = n;
  while(i-- > 0) {
    *((uint8_t*)dst++) = *((uint8_t*)src++);
  }
}
// FIXME: memmove breaks for overlapping areas
void memmove(void *dst, void *src, size_t n) {
  size_t i = n;
  while(i-- > 0) {
    *((uint8_t*)dst++) = *((uint8_t*)src++);
  }
}

void rust_deallocate(void *what, size_t old_size, size_t align) {
  rust_allocd -= old_size;
  cor_printk("bro, do you even free %p; %p (rust now holding %x bytes)\n", what, old_size, rust_allocd);
  // TODO
  old_size = old_size;
  align = align;
  what = what;
}

// this is not a null-terminated c string, but an u8 array of length `len`.
void rust_writek(const char *str, size_t len) {
  for(size_t i = 0; i < len; i++) {
    putc(str[i]);
  }
}

void abort() {
  cor_panic("abort() called");
}

void _Unwind_Resume() {
  cor_panic("_Unwind_Resume() called");
}
