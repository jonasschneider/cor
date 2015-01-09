void sched_init();
pid_t sched_add(void (*entry)(), const char *desc);
void sched_exec();
