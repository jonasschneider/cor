// TODO: .data and .bss sections break (probably anything besides .text)
void _start() {
  char *str = "Hello, world from init.\nI live at %p, and cor_printk is at %p.\n";
  long *lol = (long *)0x55000;

  int (*cor_printk)(const char *format, ...) = (int (*)(const char *format, ...))1337;
  int (**ptr)(const char *format, ...) = (int (**)(const char *format, ...))lol;
  cor_printk = *ptr;

  cor_printk(str, _start, cor_printk);

  __asm__ ( "hlt" );
  while(1);
}
