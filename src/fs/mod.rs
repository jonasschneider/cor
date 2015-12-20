mod cpio;

use alloc::boxed::Box;
use ::mem;
use ::virtio::{Block,Blockdev,Error as BlockError};
use core::str;
use self::cpio::{Cursor,Entry};
use collections::vec::Vec;
use collections::string::String;

#[derive(Debug)]
pub enum Error {
  ReadFailed(BlockError),
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

#[derive(Debug)]
pub struct Cpiofs {
  dev: Blockdev,
  buf: Box<[u8]>,
}

impl Cpiofs {
  pub fn new(dev: Blockdev) -> Self {
    Cpiofs { dev: dev, buf: box [0u8; 512] }
  }

  fn cursor<'t>(&'t mut self) -> Cursor<'t> {
    Cursor::new(&mut self.dev, &mut self.buf)
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
      if let Err(e) = self.dev.read(next_sector, &mut self.buf) {
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
