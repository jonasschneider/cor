//pub mod ramdev;
pub mod cached;

use core::fmt;
use alloc::boxed::Box;

#[derive(Debug)]
pub enum Error {
  InternalError,
  Unknown,
}

pub use self::cached::Cache;

pub trait Client: fmt::Debug {
  type Tag;

  // Submits a sequest to read the specified sector.
  // Returns a token that can be used to block until the read is completed.
  fn read_dispatch(&mut self, sector: u64, buf: Box<[u8]>) -> Result<Self::Tag, Error>;

  // Block until the read identified by the token is completed, then writes the read data
  // into `buf` (which must be of size 512).
  // TODO: This is actually just a badly-designed Future! We could probably just call it .wait() on the Tag?
  fn read_await(&mut self, tok: Self::Tag) -> Result<Box<[u8]>, Error>;
}
