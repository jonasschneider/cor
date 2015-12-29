#include <stdio.h>

int open(const char *path, int flags);
int read(int fd, const void *buf, size_t count);
int write(int fd, const void *buf, size_t count);

int main() {
  int fd = open("/dev/console", 0);

  while(1) {
    write(fd, "$ ", 2);

    char buf[256] = {0};
    read(fd, buf, 256);

    int pos = 0;
    while(buf[pos] != ' ' && buf[pos] != '\n') {
      pos++;
    }
    buf[pos] = 0; // set space to \0

    if(pos == 0) {
      printf("");
    } else if(buf[0] == 'e' && buf[1] == 'c' && buf[2] == 'h' && buf[3] == 'o' && buf[4] == 0) {
      printf("%s", buf+pos+1);
    } else {
      printf("sh: %s: command not found\n", buf);
    }
  }

  return 0;
}
