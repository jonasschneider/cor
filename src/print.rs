use core::prelude::*;
use core::fmt;
use core::fmt::Write;

extern {
  fn rust_writek(txt : &[u8], len: usize) -> ();
}

pub struct Kio;

impl Write for Kio {
  fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
    let sl = s.as_bytes();
    let len = s.len();

    unsafe {
      // FIXME: s is a Rust string here, but we need a C string
      rust_writek(sl, len);
    }
    Ok(()) // It's not like we care whether that worked
  }
}

// Macros

macro_rules! write {
  ($dst:expr, $($arg:tt)*) => ($dst.write_fmt(format_args!($($arg)*)))
}

macro_rules! writeln {
  ($dst:expr, $fmt:expr) => (
      write!($dst, concat!($fmt, "\n"))
  );
  ($dst:expr, $fmt:expr, $($arg:tt)*) => (
      write!($dst, concat!($fmt, "\n"), $($arg)*)
  );
}

pub fn myprint_args(fmt: fmt::Arguments) -> Result<(), fmt::Error>  {
  let kio = &mut Kio;
  let io = kio as &mut Write;
  write!(io, "{}", fmt)
}

pub fn myprintln_args(fmt: fmt::Arguments) -> Result<(), fmt::Error>  {
  let kio = &mut Kio;
  let io = kio as &mut Write;
  writeln!(io, "{}", fmt)
}

macro_rules! print {
    ($($arg:tt)*) => (::print::myprint_args(format_args!($($arg)*)))
}

macro_rules! println {
    ($($arg:tt)*) => (::print::myprintln_args(format_args!($($arg)*)))
}
