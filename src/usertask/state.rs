extern {
  pub fn trampoline_to_user();

  static mut trampoline_to_user_rip : u64;
  static mut trampoline_to_user_rsp : u64;
  static mut trampoline_to_user_codeseg : u64;

  static mut trampoline_from_user_arg1 : u64;
  static mut trampoline_from_user_arg2 : u64;
  static mut trampoline_from_user_arg3 : u64;
  static mut trampoline_from_user_arg4 : u64;

  static mut trampoline_from_user_rip : u64;
  static mut trampoline_from_user_rsp : u64;
  static mut trampoline_from_user_codeseg : u64;
}

type uptr = u64;

#[derive(Debug)]
pub struct UsermodeState {
  rip: uptr,
  rsp: uptr,
}

#[derive(Debug)]
pub enum SyscallType {
  Exit(i64),
  Write(u64, uptr, usize),
}

#[derive(Debug)]
pub enum StepResult {
  Syscall(SyscallType),
  Crash,
}

// TODO(safety): per-cpu storage of the statics
impl UsermodeState {
  pub fn new(entrypoint: u64, initial_stack: u64) -> Self {
    UsermodeState { rip: entrypoint, rsp: initial_stack }
  }

  pub fn step(&mut self) -> StepResult {
    unsafe {
      trampoline_to_user_codeseg = 24 | 3; // 3*8=GDT offset, RPL=3
      trampoline_to_user_rsp = self.rsp;
      trampoline_to_user_rip = self.rip;

      println!("Trampolining to userspace: rip@{:x} codeseg@{:x} rsp@{:x}", trampoline_to_user_rip, trampoline_to_user_codeseg, trampoline_to_user_rsp);

      trampoline_to_user();

      println!("Back from userspace! rip@{:x} codeseg@{:x} rsp@{:x}", trampoline_from_user_rip, trampoline_from_user_codeseg, trampoline_from_user_rsp);

      self.rsp = trampoline_from_user_rsp;
      self.rip = trampoline_from_user_rip;

      match trampoline_from_user_arg1 {
        1 => StepResult::Syscall(SyscallType::Exit(trampoline_from_user_arg2 as i64)),
        2 => StepResult::Syscall(SyscallType::Write(trampoline_from_user_arg2 as u64, trampoline_from_user_arg3 as uptr, trampoline_from_user_arg4 as usize)),
        _ => StepResult::Crash,
      }
    }
  }
}
