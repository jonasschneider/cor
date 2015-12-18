use core::{slice, str};

mod state;
mod elf;

extern {
  static cor_stage2_init_data: u8;
  static cor_stage2_init_data_len: usize;
}

use ::sched;
use self::state::StepResult::*;
use self::state::SyscallType::*;

pub fn exec_init() {
  println!("Starting init task! Or at least I hope so.");

  let elf: &[u8] = unsafe { slice::from_raw_parts(&cor_stage2_init_data, cor_stage2_init_data_len) };

  let loaded = unsafe { elf::load(elf) };
  println!("Load result: {:?}", loaded);

  let image = loaded.unwrap();
  let mut s = state::UsermodeState::new(image.initial_rip, image.initial_rsp);

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
