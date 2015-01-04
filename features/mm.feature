Feature: Memory management
  As a userland developer,
  I want to be able to allocate memory for my program
  So that I don't dereference invalid pointers
  TODO: This is pretty unconvincing, are there reasons
  for not doing alloc-on-page-fault [besides that sounding completely insane]?

  Scenario: Loading executable at standard address
    Given the following code for /sbin/init:
    """
      #include <stdio.h>

      int main() {
        printf("I am at ->%p<-\n", ((unsigned long)main)&(~(0x1000-1)));
      }
    """
    When I run the machine
    Then I should see "I am at ->0x400000<-"

  Scenario: Dynamic page allocation on executable load
    Given the following code for /sbin/init:
    """
      #include <stdio.h>

      int main() {
        printf("I am at ->%p<-\n", ((unsigned long)main)&(~(0x1000-1)));
      }
    """
    And I use the following linker script for init:
    """
      SECTIONS
      {
        . = 0x13000;
        .text : { *(.text) }
        . = 0x14000;
        .data : { *(.data) }
        .bss : { *(.bss) }
      }
    """
    When I run the machine
    Then I should see "I am at ->0x13000<-"

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
