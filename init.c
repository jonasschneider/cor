

void _start() {
  char *str = "Hello, world from kernel-space loaded init (i'm not even kidding..)\n";
  int (*cor_printk)(const char *format, ...);
  cor_printk = (int (* )(const char *format, ...))0x000000000001048a;
  cor_printk(str-0x400000+0x70000);

  __asm__ ( "hlt" : : "a" (str) );
  while(1);
}
