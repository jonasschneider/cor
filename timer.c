#include "common.h"
#include "io.h"
#include "timer.h"

void timer_init() {
  // Reference: http://wiki.osdev.org/Programmable_Interval_Timer
  // apparently this means:
  // channel 0, lobyte/hibyte, rate generator
  cor_outb(0b00110100, 0x43);

  // Now set the reload value. This determines the length of the interval between
  // the timer firing. (That means 0xffff is the slowest)
  uint16_t reload = TIMER_RELOAD;

  cor_outb((unsigned char)reload, 0x40);
  cor_outb((unsigned char)(reload>>8), 0x40);

  cor_printk("ticking at ~%u hz.. ",(uint32_t)TIMER_HZ); // print doesn't support floats yet
}
