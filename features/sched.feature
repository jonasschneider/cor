Feature: Kernelspace multitasking
  Scenario: Two simple concurrent threads
    Given the following code in the kernel:
      """
      void thread1() {
        while(1) {
          cor_printk("Hello from t2! Working for a bit...\n", stack);
          for(int i=0;i<100000000;i++);
          kyield();
        }
      }

      void thread2() {
        while(1) {
          cor_printk("Hello from t2! Working for a bit...\n", stack);
          for(int i=0;i<100000000;i++);
          kyield();
        }
      }
      """
    And I configure the kernel to start a thread with entrypoint "thread1"
    And I configure the kernel to start a thread with entrypoint "thread2"
    When I run the machine
    Then I should see "Hello from t1!" alternating with "Hello from t2!"
