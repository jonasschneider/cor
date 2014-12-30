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

struct elf64_sectionheader {
  uint32_t name;
  uint32_t type;
  uint64_t flags;
  uint64_t addr;
  uint64_t offset;
  uint64_t size;
  uint32_t link;
  uint32_t info;
  uint64_t addralign;
  uint64_t entsize;
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

  if(sizeof(struct elf64_sectionheader) != hdr->sh_entsize) {
    cor_printk("ERROR: ELF section header entry size invalid\n");
    return -1;
  }

  cor_printk("Nsections: %x\n", hdr->sh_entnum);

  if(hdr->sh_entnum > 30) {
    cor_printk("ELF: too many sections");
    return -1;
  }

  int shift = 0x382000;

  for(int i = 0; i < hdr->sh_entnum; i++) {
    void *addr = target + hdr->sh_offset + hdr->sh_entsize * i;
    struct elf64_sectionheader *sectionheader = (struct elf64_sectionheader *) addr;

    char *target_base = (char *)sectionheader->addr-shift;

    cor_printk("segment %x at %x starts at %x shifted to %x, off %x, sz %x\n",i,addr, sectionheader->addr, target_base, sectionheader->offset, sectionheader->size);
    if(sectionheader->addr > 0) {
      for(int j = 0; j < sectionheader->size; j++) {
        char *loadtarget = target_base+j;
        char *loadsrc = (char *)target + sectionheader->offset + j;
        if(loadtarget == (char*)0xe010c) {
          cor_printk("Shoving %x (%x) -> %x (%x) ..", loadsrc, *loadsrc, loadtarget, *loadtarget);
        }
        *loadtarget = *loadsrc;
        if(loadtarget == (char*)0xe010c) {
          cor_printk(".. now it's %x\n" ,*loadtarget);
        }
      }
    }
  }

  void (*entry)() = (void(*)(void *))(hdr->entrypoint - shift);

  cor_printk("will now jump to %x (%x)\n",entry, *(char*)(hdr->entrypoint - shift));

  entry();

  cor_printk("done?!\n");
  return 0;
}
