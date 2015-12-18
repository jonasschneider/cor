use core::{slice, str};

mod state;

extern {
  pub fn cor_load_init() -> u64;
}

use ::sched;
use self::state::StepResult::*;
use self::state::SyscallType::*;

pub fn exec_init() {
  println!("Starting init task! Or at least I hope so.");

  // FIXME: stack!
  let mut s = state::UsermodeState::new(unsafe { cor_load_init() }, 0x602000);

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
        println!("unknown syscall: {:?}", r);
      }
    }
    sched::kyield();
  }

  // FIXME FIXME: this is superbad; better than overwriting kernel code, but still bad
  //uint64_t rsp = (uint64_t)t->brk;
}
