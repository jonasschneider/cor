
use core::prelude::*;
use core::fmt;


macro_rules! write {
    ($dst:expr, $($arg:tt)*) => ((&mut *$dst).write_fmt(format_args!($($arg)*)))
}

macro_rules! writeln {
    ($dst:expr, $fmt:expr $($arg:tt)*) => (
        write!($dst, concat!($fmt, "\n") $($arg)*)
    )
}

pub fn myprint_args(fmt: fmt::Arguments) -> Result<(), fmt::Error>  {
  let kio = &mut Kio { lol: 1337 };
  let io = kio as &mut fmt::Writer;
  write!(io, "{}", fmt)
}

pub fn myprintln_args(fmt: fmt::Arguments) -> Result<(), fmt::Error>  {
  let kio = &mut Kio { lol: 1337 };
  let io = kio as &mut fmt::Writer;
  writeln!(io, "{}", fmt)
}

macro_rules! print {
    ($($arg:tt)*) => (print::myprint_args(format_args!($($arg)*)))
}

macro_rules! println {
    ($($arg:tt)*) => (print::myprintln_args(format_args!($($arg)*)))
}

extern {
  fn rust_writek(txt : &[u8], len: uint) -> ();
}

struct Kio {
  lol: int,
}

impl fmt::Writer for Kio {
  fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
    let sl = s.as_bytes();
    let len = s.len();

    unsafe {
      // FIXME: s is a Rust string here, but we need a C string
      rust_writek(sl, len);
    }
    Ok(()) // yes, we're lying
  }
}
