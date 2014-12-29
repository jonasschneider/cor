#include "printk.h"
#include "common.h"

#pragma pack(push, 1)
struct elf64_header {
  uint32_t magic;
  uint8_t class;
  uint8_t endian;
  uint8_t version1;
  uint8_t abi_major;
  uint8_t abi_minor;
  uint32_t pad1;
  uint16_t pad2;
  uint8_t pad3;
  uint16_t type;
  uint16_t arch;
  uint32_t version2;
  uint64_t entrypoint;
  uint64_t ph_offset;
  uint64_t sh_offset;
  uint32_t flags;
  uint16_t mysize;
  uint16_t ph_entsize;
  uint16_t ph_entnum;
  uint16_t sh_entsize;
  uint16_t sh_entnum;
  uint16_t sh_section_name_entry_idx;
};
#pragma pack(pop)

int cor_elf_exec(char *data, int len) {
  cor_printk("Called cor_elf_exec with data=%x, len=%x\n",data, len);
  if(sizeof(struct elf64_header) != 64) {
    cor_printk("assertion failed: elf64 header struct is size %x\n",sizeof(struct elf64_header));
    return -1;
  }

  char *target;
  target = (char*)0x70000;
  cor_printk("Copying to %x\n",target);

  // TODO: memcpy
  for(int i = 0; i < len; i++) {
    target[i] = data[i];
  }

  cor_printk("Wrote ELF there.\n",target);

  struct elf64_header *hdr = (struct elf64_header *)target;

  if(0x464c457f != hdr->magic) {
    cor_printk("ERROR: magic of %x did not match ELF header of 0x464c457f\n", hdr->magic);
    return -1;
  } else {
    cor_printk("ELF magic looks OK.\n");
  }


  cor_printk("done?!\n");
  return 0;
}
