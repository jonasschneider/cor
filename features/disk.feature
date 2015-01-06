Feature: Disk access
  In order to permanently store data and read it back again
  As a kernel user
  I want to be able to access hard disks attached to the system

  Scenario: Reading from a virtio block device
    Given I have a disk image with a sector full of "OHAI THERE"
    And I attach this image as a virtio block device
    When I run the machine
    Then I should see "OHAI THERE"
