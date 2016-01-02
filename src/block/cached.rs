use prelude::*;

use super::{Client, Error};

pub struct SectorCheckout {
  _data: Box<[u8]>,
}

impl Deref for SectorCheckout {
  type Target = [u8];
  fn deref(&self) -> &[u8] {
    &self._data[..]
  }
}

//impl Drop for SectorCheckout -> return buffer to cache .. maybe just an Arc?

// idea: page cache returns Page objects that contain a SleepingRWLock on the page memory?
// TODO: Cache should be Clone
pub trait Cache: Sync + fmt::Debug {
  // Read the specified sector from the page cache.
  // Will block the current process on a cache miss.
  // The returned Sector acts like a [u8; 512]. When the Sector is dropped,
  // the buffer is returned to the cache.
  fn get(&self, sector: u64) -> Result<SectorCheckout, Error>;
}


unsafe impl<C: Client> Sync for NoopCache<C> {} // TODO TODO
#[derive(Debug)]
pub struct NoopCache<C: Client> {
  blockdev: C,
}

impl<C: Client> Cache for NoopCache<C> {
  fn get(&self, sector: u64) -> Result<SectorCheckout, Error> {
    // just dumb read & block immediately
    let b = vec![0u8;512].into_boxed_slice();
    let tok = self.blockdev.read_dispatch(sector as u64, b).unwrap();
    let buf = self.blockdev.read_await(tok).unwrap();
    Ok(SectorCheckout{ _data: buf })
  }
}

// didn't we say Client was sync and shared-not-cloned? idk
impl<C: Client> NoopCache<C> {
  pub fn new(c: C) -> Self {
    NoopCache{ blockdev: c }
  }
}
