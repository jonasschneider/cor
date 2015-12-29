#include <stdio.h>

int open(const char *path, int flags);
int read(int fd, const void *buf, size_t count);
int write(int fd, const void *buf, size_t count);

int main() {
  int fd = open("/dev/console", 0);

  while(1) {
    write(fd, "> ", 2);

    char buf[256] = {0};
    read(fd, buf, 256);

    printf("%s\n", buf);
  }

  return 0;
}
