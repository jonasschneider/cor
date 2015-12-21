use super::{Error,Client,ReadWaitToken};
use alloc::boxed::Box;
use collections::vec::Vec;
use sync::global_mutex::GlobalMutex;
use core::slice::bytes::copy_memory;

// this could be SleepingMutex as well.
#[derive(Debug)]
pub struct Ramdev(GlobalMutex<Vec<u8>>);

impl Client for Ramdev {
  fn read_dispatch(&self, sector: u64) -> Result<ReadWaitToken, Error> { Ok(sector) }

  fn read_await(&self, tok: ReadWaitToken, mut buf: &mut [u8]) -> Result<(), Error> {
    // TODO: bounds check
    assert_eq!(buf.len(), 512);
    let offs = ((tok as u64) * 512) as usize;
    let mem = self.0.lock();
    copy_memory(&mem[offs..offs+512], &mut buf); // this could be a short read
    Ok(())
  }
}

impl Ramdev {
  pub fn new_striped(n_sectors: usize) -> Ramdev {
    let mut mem = Vec::with_capacity(n_sectors*512);
    for i in 0..n_sectors {
      let data = [(i % 256) as u8; 512];
      copy_memory(&data, &mut mem[(i*512)..(i*512)+512]);
    }
    Ramdev(GlobalMutex::new(mem))
  }
}
