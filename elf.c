#include "printk.h"

int cor_elf_exec(char *data, int len) {
  cor_printk("Called cor_elf_exec with data=%x, len=%x\n",data, len);

  char *target;
  target = (char*)0x70000;
  cor_printk("Putting it at %x\n",target);

  // TODO: memcpy
  for(int i = 0; i < len; i++) {
    target[i] = data[i];
  }

  cor_printk("Wrote ELF there.",target);
  int actual_magic = *((int*)target);
  if(0x464c457f != actual_magic) {
    cor_printk("ERROR: magic of %x did not match ELF header of 0x464c457f\n");
    return -1;
  }

  cor_printk("done?!\n");
  return 0;
}
