#include "common.h"

#pragma pack(push, 1)
struct tss
{
   uint32_t reserved1;
   uint64_t rsp0;
   uint64_t rsp1;
   uint64_t rsp2;
   uint64_t reserved2;
   uint64_t ist1;
   uint64_t ist2;
   uint64_t ist3;
   uint64_t ist4;
   uint64_t ist5;
   uint64_t ist6;
   uint64_t ist7;
   uint64_t reserved3;
   uint16_t reserved4;
   uint16_t iomap_base;
};
#pragma pack(pop)

void tss_setup()
{
   // The corresponding GDT entry is set up by boot.s
   struct tss *my_tss = (struct tss *)(0x80000|0x0000008000000000);
   my_tss->rsp0 = 0x55000|0x0000008000000000;
   // TODO: IST whatup?

   __asm__ (
      "mov $40, %ax\n"
      "ltr %ax"
      );
}
