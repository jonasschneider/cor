typedef unsigned long long uint64_t;

size_t rust_allocd = 0;

uint64_t corlib_alignoverride = 0;

void *tkalloc(size_t s, const char *d, size_t align);
void putc(char);
void cor_printk(const char *f, ...);
void cor_panic(const char *x);

// TODO: rust actually has these as uint, not size_t... bad?
void *rust_allocate(size_t size, size_t align) {
  if(corlib_alignoverride != 0) {
    align = corlib_alignoverride;
  }
  rust_allocd += size;
  align = align;
  void *ptr = tkalloc(size, "rust_allocate()", align);
  void *x = ptr;
  while(x < ptr) {
    *((unsigned char*)x++) = 0;
  }
  return ptr;
}

void rust_deallocate(void *what, size_t old_size, size_t align) {
  rust_allocd -= old_size;
  cor_printk("[free %p]", what);
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
