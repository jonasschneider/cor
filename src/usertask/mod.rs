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

//type Fd = u64;
//type Fdt = Map<Fd, Arc<GlobalMutex<RefCell<File>>>>;

pub fn exec_init() {
  println!("Starting init task! Or at least I hope so.");

  let serport = unsafe { cpuio::alloc(0xc080, 24, "XXXXXXXXXXXXXXXXXXXXXXXX").unwrap() };
  let mut serdev = virtio::serial::Serialdev::new(serport).unwrap();
  // serdev.putc('.');
  // serdev.putc('\n');
  // panic!("done");

  // TODO: request this from somewhere
  let port = unsafe { cpuio::alloc(0xc040, 24, "XXXXXXXXXXXXXXXXXXXXXXXX").unwrap() };

  let blockdev = virtio::block::Blockdev::new(port).unwrap();
  println!("result of blockdevice init: {:?}", blockdev);

  let cache = block::cached::NoopCache::new(blockdev);

  let mut fs = fs::Cpiofs::new(Arc::new(cache) as Arc<block::Cache>);
  println!("fs: {:?}", fs);


  println!("||\n||  $ ls");
  for x in fs.index("/").unwrap() {
    println!("||  {}", x);
  }

  let size = fs.stat("/init").unwrap();
  println!("||\n||  $ stat /init");
  println!("||  size {}", size);

  serdev.putc('*');

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

  let mut last_syscall_retval = 0;
  loop {
    let r = s.step(last_syscall_retval);
    println!("Step result: {:?}", r);

    match r {
      Syscall(Write(fd, buf, len)) => {
        let data = unsafe { slice::from_raw_parts(buf as *const u8, len as usize) }; // copy_from_user
        let text = str::from_utf8(data).unwrap();

        for c in data {
          serdev.putc(*c as char);
        }
        //print!("    | {}", text);
      },
      Syscall(Exit(ret)) => {
        println!("Init exited with 0x{:x}!", ret);
        break;
      },
      Syscall(Open(name, flags)) => {
        //let data = unsafe { slice::from_raw_parts(buf as *const u8, len as usize) }; // name_from_user
        //let text = str::from_utf8(data).unwrap();
        //print!("    | {}", text);
      },
      Syscall(Read(fd, buf, len)) => {
        let mut data = unsafe { slice::from_raw_parts_mut(buf as *mut u8, len as usize) }; // copy_from_user
        print!("before read: {:?}", data);
        last_syscall_retval = serdev.read(&mut data) as u64;
        print!("after read: {:?}", data);
        //print!("    | {}", text);
      },
      // Syscall(s) => {
      //   println!("syscall: {:?}",s);
      // },
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
