use super::virtio::{Block,Blockdev,Error as BlockError};

#[derive(Debug)]
enum Error {
  ReadFailed(BlockError),
  InvalidDiskFormat,
  Unknown,
  NotFound,
}

use alloc::boxed::Box;
use super::mem;
use core::str;

pub trait Fs<'t> {
  fn stat(&mut self, name: &str) -> Result<usize, Error>;
  fn slurp(&mut self, name: &str, buf: &mut[u8]) -> Result<usize, Error>;

  fn open<'u: 't>(&'u self, name: &str) -> Result<File<'u>, Error>;
  fn index(&mut self, dirname: &str) -> Result<Vec<String>, Error>;
}

#[derive(Debug)]
pub struct Arfs {
  dev: Blockdev,
  buf: Box<[u8]>,
}

impl Arfs {
  pub fn new(dev: Blockdev) -> Self {
    Arfs { dev: dev, buf: box [0u8; 512] }
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

#[derive(Debug)]
struct File<'f> {
  fs: &'f Arfs
}

#[derive(Debug)]
struct Entry {
  name: String,
  size: usize,
  pos: (usize, usize),
}

struct Cursor<'t> {
  dev: &'t mut Blockdev,

  buf: &'t mut [u8],
  loaded: usize,

  // as (sector, offset)
  next_header: (usize, usize),
}

use collections::string::ToString;

impl<'t> Iterator for Cursor<'t> {
  // On error, you can retry or break.
  type Item = Result<Entry, Error>;

  fn next(&mut self) -> Option<Result<Entry, Error>> {
    println!("Trying to read header at {:?}", self.next_header);
    let (sector, offset) = self.next_header;
    if self.loaded != sector {
      if let Err(e) = self.dev.read(sector, &mut self.buf) {
        return Some(Err(Error::ReadFailed(e)));
      }
      self.loaded = sector;
    }

    //println!("sector {}: {:?}", self.loaded, &self.buf[..]);

    let entry = &mut self.buf[offset..];
    //println!("entry: {:?}", &entry);
    let magic = (entry[0] as u16) | ((entry[1] as u16)<<8);
    if magic != 0o70707 {
      return Some(Err(Error::InvalidDiskFormat));
    }

    let mut namelength = ((entry[20] as u16) | ((entry[21] as u16)<<8)) as usize;

    // nice byte order, bro..
    let size = (((entry[22] as u32)<<16) | ((entry[23] as u32)<<24) | (entry[24] as u32) | ((entry[25] as u32)<<8)) as usize;

    // we'll panic here if we hit a sector boundary
    let name = match str::from_utf8(&&entry[26..(26+namelength-1)]) {
        Ok(v) => v,
        Err(e) => return Some(Err(Error::Unknown))
    };

    // Break if we found the end marker.
    if name.as_bytes() == "TRAILER!!!".as_bytes() {
      return None;
    }

    // The name and body blobs are u16-padded.
    let mut next_offset = offset + 26 + namelength;
    if next_offset & 1 != 0 {
      next_offset += 1;
    }
    next_offset += size;
    if next_offset & 1 != 0 {
      next_offset += 1;
    }

    self.next_header = (sector + next_offset / 512, next_offset % 512);

    Some(Ok(Entry{ pos: (sector, offset), size: size, name: name.to_string() }))
  }
}

impl Arfs {
  fn cursor<'t>(&'t mut self) -> Cursor<'t> {
    Cursor{ dev: &mut self.dev, buf: &mut self.buf, next_header: (0,0), loaded: !0 as usize }
  }
}

use collections::vec::Vec;
use collections::string::String;

impl<'t> Fs<'t> for Arfs {
  fn open<'u: 't>(&'u self, name: &str) -> Result<File<'u>, Error> {
    Ok(File{ fs: &self })
  }

  fn index(&mut self, dirname: &str) -> Result<Vec<String>, Error> {
    //let filename_needle = &dirname[1..dirname.len()]; // strip off leading '/'

    let mut s: Vec<String> = vec![];

    for entry in self.cursor() {
      if let Ok(e) = entry {
        s.push(e.name);
      }
    }

    Ok(s)
  }

  fn stat(&mut self, filename: &str) -> Result<usize, Error> {
    let filename_needle = &filename[1..filename.len()]; // strip off leading '/'

    for entry in self.cursor() {
      if let Ok(e) = entry {
        if e.name.as_bytes() == filename_needle.as_bytes() {
          return Ok(e.size);
        }
      }
    }

    Err(Error::NotFound)
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
