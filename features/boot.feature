Feature: Booting /sbin/init
  Scenario: Booting Hello, World
    Given the following code for /sbin/init:
      """
      #include <stdio.h>

      int main() {
        printf("Hello, world from userspace!\n");
        return 0;
      }
      """
    When I run the machine
    Then I should see "Hello, world from userspace!\n"
