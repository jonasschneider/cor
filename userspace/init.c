#include <stdio.h>

int main() {
  printf("I am at ->%p<-\n", ((unsigned long)main)&(~(0x1000-1)));
}