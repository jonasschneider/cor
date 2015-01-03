#include <stdio.h>

int main() {
  int *ptr_one;

  ptr_one = (int *)malloc(sizeof(int));

  if (ptr_one == 0) {
    printf("malloc 1 failed\n");
    return 1;
  }

  *ptr_one = 25;

  printf("The number is ->%d<-\n", *ptr_one);

  //free(ptr_one);

  return 0;
}