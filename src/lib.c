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

void rust_printk(const char *str) {
  cor_printk(str);
}

// this is not a null-terminated c string, but an u8 array of length `len`.
void rust_writek(const char *str, size_t len) {
  for(size_t i = 0; i < len; i++) {
    putc(str[i]);
  }
}

void abort() {
  while(1);
}

void _Unwind_Resume() {
  cor_panic("rust's _Unwind_Resume() got called!");
}

// What follows is stuff that's needed when we don't --gc-sections out all
// the parts of the rust object file that we don't use.
// Since we do that, I'll just keep them here for reference.


void memcpy() {}
void memset() {}
void memcmp() {}
void fmodf() {}
void fmod() {}

/*
// http://linux.die.net/man/3/memset

// TODO: i think we should just make all of these be bottom

// TODO: __thread is going to break hard, actually.
__thread int errno;


void fwrite() {

}

void __assert_fail() {

}


void *memchr(const void *a, int b, size_t c) {
  return __builtin_memchr(a, b, c);
}
int      memcmp(const void *a, const void *b, size_t c) {
  return __builtin_memcmp(a, b, c);
}
void *memcpy(void *a, const void *b, size_t c) {
  return __builtin_memcpy(a, b, c);
}
void *memmove(void *a, const void *b, size_t c) {
  return __builtin_memmove(a, b, c);
}

void *memset(void *s, int c, size_t n) {
  return __builtin_memset(s,c,n);
}


void fflush() {

}
void fprintf() {

}
void fputs() {

}

void __get_cpu_features() {

}

void stderr() {

}
void _Unwind_GetIP() {

}
void _Unwind_GetLanguageSpecificData() {

}
void _Unwind_GetRegionStart() {

}
void _Unwind_SetGR() {

}
void _Unwind_SetIP() {

}
*/
