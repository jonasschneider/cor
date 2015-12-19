#include "../common.h"

// TODO: rust actually has these as uint, not size_t... bad?
void *rust_allocate(size_t size, size_t align) {
  return tkalloc(size, "rust_allocate()", align);
}

// TODO: use the compiler intrinsics
void memmove(void *dest, const void *source, size_t n) {
  size_t i;

  uint8_t *dest8 = (uint8_t *)dest;
  uint8_t *source8 = (uint8_t *)source;
  for (i = 0; i < n; i++) {
    dest8[i] = source8[i];
  }
}

void rust_deallocate(void *what, size_t old_size, size_t align) {
  cor_printk("bro, do you even free %p; %p\n", what, old_size);
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
