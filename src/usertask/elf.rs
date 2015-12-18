use core::{mem,slice};
use core::ptr::copy;
use core::iter::{Iterator,IntoIterator};

use super::super::mem::*;

#[derive(Debug)]
pub struct Image {
  pub initial_rsp: usize,
  pub initial_rip: usize,
}

#[derive(Debug)]
pub enum Error {
  InvalidElf
}

macro_rules! ensure {
  ($x:expr) => (if !$x { return Err(Error::InvalidElf)} );
}

type task = u8; // sigh

extern {
  fn task_new() -> *mut task;
  fn task_addpage(t: *mut task, page: *const task);
  fn task_enter_memspace(t: *mut task);
}

// Unsafe because we don't respect the address space at all.
// TODO: actually store which segments and pages we allocated.
pub unsafe fn load(elf: &[u8]) -> Result<Image, Error> {
  println!("Loading Elf from {:p}, len {}", elf.as_ptr(), elf.len());

  assert_eq!(64, mem::size_of::<Elf64Header>());
  ensure!(elf.len() >= mem::size_of::<Elf64Header>());

  let hdr: &Elf64Header = mem::transmute(elf.as_ptr());

  if hdr.magic != 0x464c457f {
    println!("ERROR: magic value {:x} did not match ELF header of 0x464c457f", hdr.magic);
    return Err(Error::InvalidElf);
  } else {
    println!("ELF magic looks OK.");
  }

  ensure!(hdr.sh_entsize as usize == mem::size_of::<Elf64SectionHeader>());

  let first_section_off = hdr.sh_offset as usize;
  let n_sections = hdr.sh_entnum as usize;

  ensure!(0 < n_sections);
  ensure!(n_sections < 30);

  let task = task_new();

  ensure!(elf.len() >= first_section_off
            + n_sections * mem::size_of::<Elf64SectionHeader>());

  let sections: &[Elf64SectionHeader] = unsafe {
    slice::from_raw_parts(mem::transmute(elf.as_ptr().offset(first_section_off as isize)), n_sections)
  };

  for s in sections.iter().filter(|s| s.addr != 0 && s.size != 0) {
    println!("Found a nonempty section: [{:x}; {:x}] {:?}", s.addr, s.size, s);

    let startpage = align_down(s.addr as usize, 0x1000); // align down
    let endpage = align_up((s.addr+s.size) as usize, 0x1000); // align up
    println!("It needs these pages: {:x} - {:x}", startpage, endpage);

    assert!(endpage > startpage); // empty sections are disallowed

    let mut page = startpage;
    while page < endpage {
      println!("Adding page at {:x}", page);

      // TODO: I believe task_addpage filters duplicates, but unsure
      task_addpage(task, page as *const u8);
      page += 0x1000;
    }
  }

  task_enter_memspace(task); // This is super unsafe!

  let mut brk = 0;

  for section in sections.iter().filter(|s| s.addr != 0 && s.size != 0) {
    if section._type == 0x8 && section.flags == 0x3 {
      // This seems to indicate the .data section, i.e. in the traditional memory layout,
      // the program break will be at the end of this section.
      // Our current, fairly ridiculous setup is to use this break location as the initial stack.
      // This means we will inevitably write into our own data. But we'll get to that later.

      // The least we can do is round up to the nearest page bound to cut us some slack.
      // (We allocated the entire page anyway in the above loop)
      brk = align_up((section.addr + section.size) as usize, 0x1000);
      println!("Found data section at {:x}, setting brk to {:x}", section.addr, brk);
    }

    ensure!(elf.len() >= (section.offset as usize) + (section.size as usize));
    let data_source = elf.as_ptr().offset(section.offset as isize) as *const u8;
    let data_dest = section.addr as *mut u8;

    copy(data_source, data_dest, section.size as usize);
  }

  // We should have found the break.
  ensure!(brk != 0);

  // Memory sanity check: This should be the opcode for "push %rbp", the first instruction
  // in _start. Actually a horrible way to check this, but meh.
  let firstword = hdr.entrypoint as *const u16;
  ensure!(*firstword == 0x4855);

  Ok(Image{initial_rip: hdr.entrypoint as usize, initial_rsp: brk})
}

#[derive(Debug)]
#[repr(C, packed)]
struct Elf64Header {
  magic: u32,
  class: u8,
  endian: u8,
  version1: u8,
  abi_major: u8,
  abi_minor: u8,
  pad1: u32,
  pad2: u16,
  pad3: u8,
  _type: u16,
  arch: u16,
  version2: u32,
  entrypoint: u64,
  ph_offset: u64,
  sh_offset: u64,
  flags: u32,
  mysize: u16,
  ph_entsize: u16,
  ph_entnum: u16,
  sh_entsize: u16,
  sh_entnum: u16,
  sh_section_name_entry_idx: u16,
}

#[derive(Debug)]
#[repr(C, packed)]
struct Elf64SectionHeader {
  name: u32,
  _type: u32,
  flags: u64,
  addr: u64,
  offset: u64,
  size: u64,
  link: u32,
  info: u32,
  addralign: u64,
  entsize: u64,
}
