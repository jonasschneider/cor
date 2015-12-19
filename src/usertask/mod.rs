use core::{slice, str};

mod state;
mod elf;

extern {
  static cor_stage2_init_data: u8;
  static cor_stage2_init_data_len: usize;
}

use super::{virtio,cpuio,fs};
use fs::Fs;

use ::sched;
use self::state::StepResult::*;
use self::state::SyscallType::*;

pub fn exec_init() {
  println!("Starting init task! Or at least I hope so.");
  let static_elf: &[u8] = unsafe { slice::from_raw_parts(&cor_stage2_init_data, cor_stage2_init_data_len) };
  {
    let port = cpuio::alloc(0xc040, 24).unwrap(); // 24 pins, see virtio spec
    let blockdev = unsafe { virtio::init(port) }.unwrap();
    println!("result of blockdevice init: {:?}", blockdev);

    let mut fs = fs::Arfs::new(blockdev);

    let mut buf = vec!(0u8; cor_stage2_init_data_len+10);
    let read = fs.read("/init", &mut buf);
    println!("Read result: {:?}",read);
    let n = read.unwrap();
    println!("Lengths: expected {}, actual {}",cor_stage2_init_data_len,n);
    assert_eq!(cor_stage2_init_data_len, n);

    let loaded_elf = &buf[0..n];

    let mut i = 0;
    while i < n {
      if loaded_elf[i] != static_elf[i] {
        println!("at {}: read: {:?}, static: {:?}", i, &loaded_elf[i..i+20], &static_elf[i..i+20]);
        break;
      }
      i = i + 1;
    }


    assert_eq!(static_elf, loaded_elf);

    println!("it works!");
    panic!();
  }

  let loaded = unsafe { elf::load(static_elf) };
  println!("Load result: {:?}", loaded);

  let image = loaded.unwrap();
  let mut s = state::UsermodeState::new(image.initial_rip as u64, image.initial_rsp as u64);

  loop {
    let r = s.step();
    println!("Step result: {:?}", r);
    match r {
      Syscall(Write(fd, buf, len)) => {
        let data = unsafe { slice::from_raw_parts(buf as *const u8, len as usize) };
        let text = str::from_utf8(data).unwrap();
        print!("    | {}", text);
      },
      Syscall(Exit(ret)) => {
        println!("Init exited with 0x{:x}!", ret);
        break;
      },
      _ => {
        println!("unknown syscall, crashing process: {:?}", r);
        break
      }
    }
    sched::kyield();
  }

  println!("User process exited normally or due to crash.");

  // FIXME FIXME: this is superbad; better than overwriting kernel code, but still bad
  //uint64_t rsp = (uint64_t)t->brk;
}
