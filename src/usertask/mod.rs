use core::{slice, str};

mod state;
mod elf;

use drivers::virtio;
use super::{cpuio,fs};
use fs::Fs;

use block;
use sched;
use core::cell::UnsafeCell;
use self::state::StepResult::*;
use self::state::SyscallType::*;
use alloc::arc::Arc;

pub fn exec_init() {
  println!("Starting init task! Or at least I hope so.");

  // TODO: request this from t
  let port = unsafe { cpuio::alloc(0xc040, 24, "XXXXXXXXXXXXXXXXXXXXXXXX").unwrap() };
  let blockdev: virtio::Blockdev = virtio::Blockdev::new(port).unwrap();
  println!("result of blockdevice init: {:?}", blockdev);

  let mut fs = fs::Cpiofs::new(Arc::new(blockdev) as Arc<block::Client>);
  println!("fs: {:?}", fs);


  println!("||\n||  $ ls");
  for x in fs.index("/").unwrap() {
    println!("||  {}", x);
  }

  let size = fs.stat("/init").unwrap();
  println!("||\n||  $ stat /init");
  println!("||  size {}", size);

  let mut buf = vec!(0u8; size);
  let read = fs.slurp("/init", &mut buf);

  {
    let file1 = fs.open("/init");
    let file2 = fs.open("/README.md");

    println!("Files 1 and 2: {:?} {:?}",file1, file2);
  }

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
