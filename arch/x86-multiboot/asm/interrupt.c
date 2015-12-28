#include "common.h"
#include "cor/syscall.h"

// see interrupthandler.s
void intrstub_0();
void timer_isr();

#pragma pack(push, 1)
struct {
  uint16_t limit;
  uint64_t base;
} idtr;
#pragma pack(pop)

void rs_init_interrupts(void);
void interrupt_init() {
  // First, initialize the Rust IRQ dispatcher
  rs_init_interrupts();

  // Now write the IDT
  void *base = (void*)(0x6000|0x0000008000000000);
  const int entrysize = 16; // in bytes
  const int n_entry = 0x80; // 128
  idtr.base = (uint64_t)base;
  idtr.limit = entrysize * n_entry;

  for(int i = 0; i < n_entry; i++) {
    void *offset = base+(i*entrysize);
    void *target;
    if(i != 0x20) {
      void *t = (void*)&intrstub_0 + 0x10*i; // this is horrible
      target = (void*)(((ptr_t)t) | 0x0000008000000000);
    } else {
      target = (void*)(((ptr_t)&timer_isr) | 0x0000008000000000);
    }

    // cf. intel_64_software_developers_manual.pdf pg. 1832
    *(uint16_t*)(offset+0) = (uint16_t) ((uint64_t)target >> 0);
    *(uint16_t*)(offset+2) = (uint16_t) 8; // segment
    *(uint16_t*)(offset+4) = (uint16_t) 0xee00; // flags
    *(uint16_t*)(offset+6) = (uint16_t) ((uint64_t)target >> 16);
    *(uint32_t*)(offset+8) = (uint32_t) ((uint64_t)target >> 32);
    *(uint32_t*)(offset+12) = (uint32_t) 0; // reserved
  }


  void *x = (void*)&idtr;

  __asm__ (
    "lidt (%0)"
    : : "p" (x)
  );

  // Cool, now the CPU knows about our interrupt table.
  // This means that we can now fire software interrupts, like this test
  // interrupt. Also, this is the mechanism we're using to make system calls
  // from userland. There are other instructions in x86_64, like SYSCALL, that
  // are actually specialized to that now, and are likely much more efficient,
  // but doing with a software interrupt is the original way.
  debug("Firing test interrupt.. ");
  int a = 1337;
  __asm__ ( "int $48" );
  debug("returned.. ");
  if(a == 1337) {
    debug("OK, stack looks intact.\n");
  } else {
    // If you screw up, the stack pointer will be off.
    // This check is likely stupid & meaningless, since `a` will just get optimized out.
    cor_panic("Test interrupt seems to have messed up the stack.");
  }

  // Okay, from now on, we'll be able to sort-of-meaningfully handle interrupts.
  __asm__ ( "sti" );
}
