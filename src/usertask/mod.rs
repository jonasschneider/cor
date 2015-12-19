use core::{slice, str};

mod state;
mod elf;

use super::{virtio,cpuio,fs};
use fs::Fs;

use ::sched;
use self::state::StepResult::*;
use self::state::SyscallType::*;

pub fn exec_init() {
  println!("Starting init task! Or at least I hope so.");

  let port = cpuio::alloc(0xc040, 24).unwrap(); // 24 pins, see virtio spec
  let blockdev = unsafe { virtio::init(port) }.unwrap();
  println!("result of blockdevice init: {:?}", blockdev);

  let mut fs = fs::Arfs::new(blockdev);

  let size: usize = if let Ok(s) = fs.stat("/init") {
    s
  } else {
    panic!("failed!");
  };
  println!("Init size: {:?}", size);

  let mut buf = vec!(0u8; size+1000);
  let read = fs.read("/init", &mut buf);
  println!("Read result: {:?}",read);
  let n = read.unwrap();
  println!("Succesfully read init from disk.");

  let loaded_elf = &buf[0..n];

  let loaded = unsafe { elf::load(loaded_elf) };
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
