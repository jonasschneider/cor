Feature: Disk access
  In order to permanently store data and read it back again
  As a kernel user
  I want to be able to access hard disks attached to the system

  Scenario: Reading from a virtio block device
    Given I have a disk image with a sector full of "OHAI THERE"
    And I attach this image as a virtio block device
    When I run the machine
    Then I should see "OHAI THERE"

  Scenario: Reading from a file on the boot disk
    Given I have a boot disk containing a file "test.txt" with contents "hi from file"
    And the following code for init:
      """
      #include <stdio.h>

      int main() {
        size_t ret_in;
        char buffer[24];

        int fd = open("/test.txt, O_RDONLY);
        if (fd < 0) {
          perror("open");
          return 1;
        }

        ret_in = read(fd, &buffer, 24);
        if(ret_in == 12) {
          buffer[12] = 0;
          printf("in file: '%s'", buffer);
        } else {
          perror("read");
          return 1;
        }

        close (fd);
        return 0;
      }
      """
    When I run the machine
    Then I should see "in file: 'hi from file'"
