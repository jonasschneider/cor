#include "../common.h"

void *malloc(size_t siz) {
  return tkalloc(siz, "rust malloc()", 1);
}

void free(void *what) {
  // TODO
  what = what;
}

void abort() {
  while(1);
}


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
