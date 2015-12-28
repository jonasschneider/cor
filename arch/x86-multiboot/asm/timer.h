void timer_init();

#define TIMER_BASE_HZ 1193182
#define TIMER_RELOAD 0xffff
#define TIMER_HZ ((float)TIMER_BASE_HZ / TIMER_RELOAD)
