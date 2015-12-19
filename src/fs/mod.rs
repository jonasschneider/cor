use super::virtio::{Block,Blockdev,Error as BlockError};

#[derive(Debug)]
enum Error {
  ReadFailed(BlockError),
  InvalidDiskFormat,
  Unknown,
}

use super::mem;
use core::str;

pub trait Fs {
  fn read(&mut self, name: &str, buf: &mut[u8]) -> Result<usize, Error>;
}

pub struct Arfs {
  dev: Blockdev
}

impl Arfs {
  pub fn new(dev: Blockdev) -> Self {
    Arfs { dev: dev }
  }
}

// 2 magic
// 2 dev
// 2 ino
// 2 mode
// 2 uid
// 2 gid
// 2 nlink
// 2 rdev
// 4 mtime
// 2 namesize
// 4 filesize

impl Fs for Arfs {
  fn read(&mut self, filename: &str, buf: &mut[u8]) -> Result<usize, Error> {
    let filename_needle = &filename[1..filename.len()]; // strip off leading '/'
    let mut firstblock = [0u8; 512];
    if let Err(e) = self.dev.read(1, &mut firstblock) {
      return Err(Error::ReadFailed(e));
    }
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
