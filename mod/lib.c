// http://linux.die.net/man/3/memset

typedef unsigned long long size_t;

void cor_hitmarker() {
  while(1);
}

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

void abort() {

}
void fflush() {

}
void fprintf() {

}
void fputs() {

}

void __get_cpu_features() {

}
void _GLOBAL_OFFSET_TABLE_() {

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
