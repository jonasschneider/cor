#define SECTION_TEXT 1
#define SECTION_DATA 2

struct task_section {
  char type;
  void *base;
  size_t size;
  struct task_section *next;
};

struct task_table_entry {
  void *page_table_base;
  size_t brk;
  struct task_section *first_section;
};

// TODO: should the caller alloc this?
struct task_table_entry *task_new();
struct task_section *task_add_section(struct task_table_entry *t, char type, size_t size);
