// http://linux.die.net/man/3/memset

// FIXME n :: size_t
void memset(void *s, int c, int n) {
  for(int i=0; i < n; i++) {
    *((char*)(s+i)) = (char)c;
  }
}
