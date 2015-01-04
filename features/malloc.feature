Feature: Allocating memory
  As a userland developer,
  I want to be able to allocate memory for my program
  So that I don't dereference invalid pointers
  TODO: This is pretty unconvincing, are there reasons
  for not doing alloc-on-page-fault [besides that sounding completely insane]?

  @wip
  Scenario: Basic arithmetic on heap memory
    Given the following code for /sbin/init:
      """
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
      """
    When I run the machine
    Then I should see "The number is ->25<-"
