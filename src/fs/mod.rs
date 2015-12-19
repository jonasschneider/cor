use super::virtio::{Block,Blockdev,Error as BlockError};

#[derive(Debug)]
enum Error {
  ReadFailed(BlockError),
  Unknown,
}

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

impl Fs for Arfs {
  fn read(&mut self, name: &str, buf: &mut[u8]) -> Result<usize, Error> {
    let mut block = [0u8; 512];
    let res = self.dev.read(0, &mut block);
    println!("Blockdev read result: {:?}",res);
    if let Err(e) = res {
      return Err(Error::ReadFailed(e));
    }
    println!("mbr sig: {:?}", &block[510..512]);
    let res2 = self.dev.read(1, &mut block);
    println!("Blockdev read2 result: {:?}",res2);
    println!("sector1: {:?}", &block[0..2]);
    if let Err(e) = res2 {
      return Err(Error::ReadFailed(e));
    }

    Ok(0)
  }
}
