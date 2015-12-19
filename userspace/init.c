#include <stdio.h>

int main() {
  unsigned int *ptr_one;

  ptr_one = (unsigned int *)malloc(sizeof(unsigned int));

  if (ptr_one == 0) {
    printf("malloc 1 failed\n");
    return 1;
  }

  *ptr_one = 25;

  printf("The number is ->%u<-\n", *ptr_one);

  //free(ptr_one);

  return 0;
}