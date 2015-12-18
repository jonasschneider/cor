use core::{slice, str};

extern {
  pub fn cor_load_init() -> u64;

  pub fn trampoline_to_user();

  static mut trampoline_to_user_rip : u64;
  static mut trampoline_to_user_rsp : u64;
  static mut trampoline_to_user_codeseg : u64;

  static mut trampoline_from_user_arg1 : u64;
  static mut trampoline_from_user_arg2 : u64;
  static mut trampoline_from_user_arg3 : u64;
  static mut trampoline_from_user_arg4 : u64;
}

pub fn exec_init() {
  println!("Starting init task! Or at least I hope so.");

  unsafe {
    let init_entry = cor_load_init();
    if init_entry == 0 {
      println!("error while exec'ing init!");
      return;
    }
    trampoline_to_user_codeseg = 24 | 3; // 3*8=GDT offset, RPL=3
    trampoline_to_user_rsp = 0x602000; // !!
    trampoline_to_user_rip = init_entry;

    println!("Trampolining to userspace at {:x} with stack at {:x}", trampoline_to_user_rip, trampoline_to_user_rsp);

    trampoline_to_user();

    println!("Back from userspace!");
    println!("IRQ49 with args: {:x} {:x} {:x} {:x}",
      trampoline_from_user_arg1, trampoline_from_user_arg2, trampoline_from_user_arg3, trampoline_from_user_arg4);

    if trampoline_from_user_arg1 == 2 {
      let fd = trampoline_from_user_arg2;
      let buf = trampoline_from_user_arg3;
      let len = trampoline_from_user_arg4;
      println!("write() fd={:x}, buf={:x}, n={:x}", fd, buf, len);
      let data = slice::from_raw_parts(buf as *const u8, len as usize);
      let text = str::from_utf8(data).unwrap();
      print!("    | {}", text);
    }
  }

  // FIXME FIXME: this is superbad; better than overwriting kernel code, but still bad
  //uint64_t rsp = (uint64_t)t->brk;
}
