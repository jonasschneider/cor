int my_kernel_subroutine(void);

int lawl = 0xdeadbeef;

// For now, this has to be the first symbol defined or it won't be placed at 0x0. Ugh.
void kernel_main(void) {
  *((unsigned char*)0xB8000) = 'X';
  int a = 0xdead;
  int b = my_kernel_subroutine();
  int c = a + b;
  {
    int i = 0;
    while(1) {
      *((unsigned char*)0xB8000+(i++)) = 'X';
    }
  }
}

int my_kernel_subroutine() {
  return 0xbeef;
}
