Feature: PCI device detection
  In order to use my expensive peripherals
  As a kernel user
  I want to be able to detect devices connected to the system via PCI

  Background:
    When I run the machine
    Then I don't see "this is a virtio NIC"

  Scenario: Detecting a Virtio NIC
    Given I attach a virtio network interface to the machine
    When I run the machine
    Then I should see "this is a virtio NIC"
