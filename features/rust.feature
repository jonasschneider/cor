Feature: Rust kernel modules
  In order to extend my system's functionality
  While retaining speed, safety and practicality
  As a kernel developer
  I want to be able to load kernel modules written in Rust into the kernel

  @wip
  Scenario: Rust FFI
    Given the following code for a kernel module called `mod_hello`:
      """
      #[link(name = "cor")]
      extern {
          fn cor_hitmarker() -> void;
      }

      #[start]
      fn start(argc: int, argv: *const *const u8) -> int {
          cor_hitmarker();
      }
      """
    When I run the machine
    Then I should see "FIRED"

  @wip
  Scenario: Hello World using cor crate
    Given the following code for a kernel module called `mod_hello`:
      """
      extern crate cor;

      #[start]
      fn start(argc: int, argv: *const *const u8) -> int {
          cor::printk("Hello, world from my Rust module!");
          return 0;
      }
      """
    When I run the machine
    Then I should see "Hello, world from my Rust module!"

  # Maybe we also want this?
  @wip
  Scenario: Rust userspace FFI syscalls
    Given the following code for a kernel module called `mod_hello`:
      """
      #[link(name = "cor")]
      extern {
          fn exit(code: int) -> void;
      }

      */
      #[start]
      fn start(argc: int, argv: *const *const u8) -> int {
          exit(123);
      }
      """
    When I run the machine
    Then I should see "ret=123"
