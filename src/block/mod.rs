pub mod ramdev;

pub enum Error {
  Unknown,
}

// I would like to make this an associated type of Client, but then we'll have to dynamically size it,
// which sucks. If you need >64bit, you could always interpret the token as a pointer, I guess..
pub type ReadWaitToken = u64;

// The client should be threadsafe.
pub trait Client: Sync {
  // Submits a sequest to read the specified sector.
  // Returns a token that can be used to block until the read is completed.
  fn read(&self, sector: u64) -> Result<ReadWaitToken, Error>;

  // Block until the read identified by the token is completed, then writes the read data
  // into `buf` (which must be of size 512).
  fn wait_read(&self, tok: ReadWaitToken, buf: &mut [u8]) -> Result<(), Error>;
}
