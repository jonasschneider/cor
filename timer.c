#include "io.h"

void timer_init() {
  // Reference: http://wiki.osdev.org/Programmable_Interval_Timer
  // apparently this means:
  // channel 0, lobyte/hibyte, rate generator
  cor_outb(0b00110100, 0x43);

  // Now set the reload value. This determines the length of the interval between
  // the timer firing. (That means 0xffff is the slowest)
  unsigned short int reload = 0xffff;

  cor_outb((char)reload, 0x40);
  cor_outb((char)(reload>>8), 0x40);
}
