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
    let mut firstblock = [0u8; 512];
    if let Err(e) = self.dev.read(0, &mut firstblock) {
      return Err(Error::ReadFailed(e));
    }

    let entry = &firstblock[..];
    println!("entry: {:?}", &entry[..]);
    let magic = (entry[0] as u16) | ((entry[1] as u16)<<8);
    if magic != 0o70707 {
      return Err(Error::InvalidDiskFormat);
    }

    let mut namelength = ((entry[20] as u16) | ((entry[21] as u16)<<8)) as usize;
    println!("name has length {} including nul",namelength);

    // nice byte order, bro..
    let size = (((entry[22] as u32)<<16) | ((entry[23] as u32)<<24) | (entry[24] as u32) | ((entry[25] as u32)<<8)) as usize;
    println!("file size: {}, buffer length: {}",size, buf.len());

    if size > buf.len() {
      println!("Your buffer is too small, bye");
      return Err(Error::Unknown);
    }

    let read_filename = match str::from_utf8(&&entry[26..(26+namelength-1)]) {
        Ok(v) => v,
        Err(e) => return Err(Error::Unknown)
    };

    println!("Name: got '{}', wanted '{}'", read_filename, filename_needle);
    if read_filename.as_bytes() != filename_needle.as_bytes() {
      return Err(Error::Unknown);
    }

    // Due to padding, the actual start of the data is right here:
    let body_offset = mem::align_up(26+namelength, 2);

    let mut next_sector = 1;
    let mut sectorbuf = [0u8; 512];

    let mut written = 0;

    while written < size {
      let n = {
        let src = if written == 0 {
          &entry[body_offset..entry.len()]
        } else {
          &sectorbuf[..]
        };
        println!("copying, src: {:?}", &src[0..10]);
        let mut dest = &mut buf[written..size];
        dest.clone_from_slice(src)
      };

      written = written + n;
      if written < size {
        println!("Reading sector {}", next_sector);
        if let Err(e) = self.dev.read(next_sector, &mut sectorbuf) {
          return Err(Error::ReadFailed(e));
        }
        next_sector = next_sector + 1;
      }
    }

    Ok(size)
  }
}
