#define SECTION_TEXT 1
#define SECTION_DATA 2

struct task_table_entry {
  void *page_table_base;
  void *page_table_useddir;
  void *brk;
};

// TODO: should the caller alloc this?
struct task_table_entry *task_new();
int task_addpage(struct task_table_entry *t, void *page);
void task_enter_memspace(struct task_table_entry *t);
