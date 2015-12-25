pub mod ramdev;
pub mod cached;

use core::fmt;
use alloc::boxed::Box;

#[derive(Debug)]
pub enum Error {
  InternalError,
  Unknown,
}

pub use self::cached::Cache;

// I would like to make this an associated type of Client, but then we'll have to dynamically size it,
// which sucks. If you need >64bit, you could always interpret the token as a pointer, I guess..
pub type ReadWaitToken = u64;

pub trait Client: fmt::Debug {
  // Submits a sequest to read the specified sector.
  // Returns a token that can be used to block until the read is completed.
  fn read_dispatch(&self, sector: u64, buf: Box<[u8]>) -> Result<ReadWaitToken, Error>;

  // Block until the read identified by the token is completed, then writes the read data
  // into `buf` (which must be of size 512).
  // This is actually just a badly-designed future! We could probably just call it .wait() on the ReadWaitToken.
  fn read_await(&self, tok: ReadWaitToken) -> Result<Box<[u8]>, Error>;
}
