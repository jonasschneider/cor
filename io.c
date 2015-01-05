unsigned char cor_inb(unsigned short int port) {
  char res;
  __asm__ (
    "in %%dx, %%ax"
    : "=a" (res): "d" (port)
  );
  return res;
}

void cor_outb(unsigned char value, unsigned short int port) {
  __asm__ (
    "outb %%al,%%dx": :"d" (port), "a" (value)
  );
}
