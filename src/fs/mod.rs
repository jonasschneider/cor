mod cpio;

use block;
use alloc::arc::Arc;

use alloc::boxed::Box;
use ::mem;
use ::drivers::virtio::Blockdev;
use core::str;
use self::cpio::{Cursor,Entry};
use collections::vec::Vec;
use collections::string::String;

#[derive(Debug)]
pub enum Error {
  ReadFailed(block::Error),
  InvalidDiskFormat,
  Unknown,
  NotFound,
}


pub trait Fs<'t> {
  fn stat(&mut self, name: &str) -> Result<usize, Error>;
  fn slurp(&mut self, name: &str, buf: &mut[u8]) -> Result<usize, Error>;

  fn open<'u: 't>(&'u self, name: &str) -> Result<File<'u>, Error>;
  fn index(&mut self, dirname: &str) -> Result<Vec<String>, Error>;
}

/*
  note: `Mutex` will be GlobalMutex for now, but could become SleepingMutex later,
    since we don't need to access files from interrupt space.

  Every File is ultimately owned by its containing Fs. (for keeping state...?)
  The Fs only hands out a Arc<Mutex<RefCell<File>>> to the caller. Once a file is closed,
  the Fs can later reclaim it by dismantling the Arc.

  Likely, the File will own a way to allow the file access to the underlying block device.
  (A block::Client in some form.)

  Every process then has a file descriptor table of type Fdt.

  type Fd = u64;
  type Fdt = Map<Fd, Arc<GlobalMutex<RefCell<File>>>>;

  Given two processes A and B, each having opened the same File. (i.e. this is the reason for the mutex above)
  Now A wants to read from the file. The File submits an I/O request to the Blockdev
  (using a GlobalMutex<DescriptorsAndAvail> internally to control access to the vring's avail, making the Blockdev Sync).
  This request is single-sector for now, but could potentially be a range.
  The immediate return value of `enqueue`/`submit` is some kind of wait token (virtio_epoch,virtio_descriptor_id).
  A, still in the read() call on the File object,
  blocks on the completion of that wait token (maybe with a timeout?).

  The virtio IRQ owns (due to the IRQ handler's GlobalMutex) the vring's `used`.
  It checks the request IDs of returned buffers,
  and builds the wait tokens of the completed requests from them.
  (Epochs are probably going to be nasty here. Maybe we simply bump the epoch
  once we observe a bufferid smaller than the previous one.)
  `sched` uses the matching token to wake A.

  Buffer handling: Zero-copy is annoying because of block sizes and alignment.. meeeh.
  For now, the Blockdev will allocate a pool of buffers (this has the bonus of making the
  descriptor table read-only). When `turning in` the wait token, the FS gives a buffer,
  which the Virtio descriptor's buffer is copied into.

  So, in summary:
  (a) no worker threads per-file or per-filesystem
  (b) parking is on block-device level, not on FS-level (the FS operation is buried in the blocked stack)
  (c) File's need not be Sync
  (d) Blockdev's *do* need to be Sync
*/

#[derive(Debug)]
pub struct Cpiofs {
  dev: Arc<block::Client>,
  buf: Box<[u8]>,
}

impl Cpiofs {
  pub fn new(dev: Arc<block::Client>) -> Self {
    Cpiofs { dev: dev, buf: box [0u8; 512] }
  }

  fn cursor<'t>(&'t mut self) -> Cursor<'t> {
    Cursor::new(self.dev.clone(), &mut self.buf)
  }
}

#[derive(Debug)]
struct File<'f> {
  fs: &'f Cpiofs
}

impl<'t> Fs<'t> for Cpiofs {
  fn open<'u: 't>(&'u self, name: &str) -> Result<File<'u>, Error> {
    Ok(File{ fs: &self })
  }

  fn index(&mut self, dirname: &str) -> Result<Vec<String>, Error> {
    //let filename_needle = &dirname[1..dirname.len()]; // strip off leading '/'

    Ok(self.cursor().map(|e| e.unwrap().name).collect())
  }

  fn stat(&mut self, filename: &str) -> Result<usize, Error> {
    let filename_needle = &filename[1..filename.len()]; // strip off leading '/'

    match self.cursor().map(|e| e.unwrap()).find(|e| e.name.as_bytes() == filename_needle.as_bytes()) {
      Some(e) => Ok(e.size),
      None => Err(Error::NotFound),
    }
  }

  fn slurp(&mut self, filename: &str, buf: &mut[u8]) -> Result<usize, Error> {
    let filename_needle = &filename[1..filename.len()]; // strip off leading '/'
    let entry = match self.cursor().map(|e| e.unwrap()).find(|e| e.name.as_bytes() == filename_needle.as_bytes()) {
      Some(e) => e,
      None => { return Err(Error::NotFound); }
    };
    let (mut next_sector, initial_body_offset) = entry.body_pos;

    let mut written = 0;
    while written < entry.size {
      println!("Reading sector {}", next_sector);
      // just dumb read & block immediately
      let tok = self.dev.read_dispatch(next_sector as u64).unwrap();
      if let Err(e) = self.dev.read_await(tok, &mut self.buf) {
        return Err(Error::ReadFailed(e));
      }
      next_sector = next_sector + 1;

      let n = {
        let src = if written == 0 {
          &self.buf[initial_body_offset..]
        } else {
          &self.buf[..]
        };
        println!("copying, src: {:?}", &src[0..10]);
        let mut dest = &mut buf[written..entry.size];
        dest.clone_from_slice(src)
      };

      written = written + n;
    }

    assert_eq!(written, entry.size);
    Ok(written)
  }
}
