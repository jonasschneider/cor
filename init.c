void _start() {
  char *str = "Hello, world from kernel-space init\nI am loaded at %x (not even kidding..)\n";
  int (*cor_printk)(const char *format, ...);
  cor_printk = (int (* )(const char *format, ...))0x000000000001048a;

  cor_printk(str, _start);

  __asm__ ( "hlt" : : "a" (str) );
  while(1);
}
