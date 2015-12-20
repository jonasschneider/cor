use super::Error;
use ::virtio::{Block,Blockdev};
use collections::string::String;
use core::str;

// header format:
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
pub struct Entry {
  pub name: String,
  pub size: usize,
  pub pos: (usize, usize),
}

pub struct Cursor<'t> {
  dev: &'t mut Blockdev,

  buf: &'t mut [u8],
  loaded: usize,

  // as (sector, offset)
  next_header: (usize, usize),
}

impl<'t> Cursor<'t> {
  pub fn new(dev: &'t mut Blockdev, buf: &'t mut [u8]) -> Cursor<'t> {
    assert_eq!(512, buf.len());
    Cursor { dev: dev, buf: buf, next_header: (0,0), loaded: !0 as usize }
  }
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
