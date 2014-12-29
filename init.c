void _start() {
  char *str = "Hello, world from kernel-space loaded init (i'm not even kidding)\n";
  __asm__ ( "hlt" : : "a" (str) );
  while(1);
}
