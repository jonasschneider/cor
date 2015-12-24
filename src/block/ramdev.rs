use super::{Error,Client,ReadWaitToken};
use alloc::boxed::Box;
use collections::vec::Vec;
use sync::global_mutex::GlobalMutex;
use core::slice::bytes::copy_memory;

use collections::btree_map::BTreeMap;

// this could be SleepingMutex as well.
#[derive(Debug)]
pub struct Ramdev(GlobalMutex<Vec<u8>>, GlobalMutex<BTreeMap<ReadWaitToken,Box<[u8]>>>);

use core::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
static NEXT_TOKEN: AtomicUsize = ATOMIC_USIZE_INIT;

// TODO: bounds check (sector vs. length)
impl Client for Ramdev {
  fn read_dispatch(&self, sector: u64, mut buf: Box<[u8]>) -> Result<ReadWaitToken, Error> {
    assert_eq!(buf.len(), 512);

    assert_eq!(buf.len(), 512);
    let offs = ((sector as u64) * 512) as usize;
    let mem = self.0.lock();
    copy_memory(&mem[offs..offs+512], &mut buf); // this could be a short read

    let mut map = self.1.lock();
    let tok = NEXT_TOKEN.fetch_add(1, Ordering::SeqCst) as ReadWaitToken;
    map.insert(tok, buf);

    Ok(tok)
  }

  fn read_await(&self, tok: ReadWaitToken) -> Result<Box<[u8]>, Error> {
    let mut map = self.1.lock();
    let buf = map.remove(&tok).unwrap(); // panic on invalid, for now
    Ok(buf)
  }
}

impl Ramdev {
  pub fn new_striped(n_sectors: usize) -> Ramdev {
    let mut mem = Vec::with_capacity(n_sectors*512);
    for i in 0..n_sectors {
      let data = [(i % 256) as u8; 512];
      copy_memory(&data, &mut mem[(i*512)..(i*512)+512]);
    }
    Ramdev(GlobalMutex::new(mem), GlobalMutex::new(BTreeMap::new()))
  }
}
