Feature: Disk access
  In order to permanently store data and read it back again
  As a kernel user
  I want to be able to access hard disks attached to the system

  Scenario: Reading from a virtio block device
    Given I have a disk image with a sector full of "OHAI THERE"
    When I run the machine with this disk image attached as a virtio block device
    Then I should see "OHAI THERE"
