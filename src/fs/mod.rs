use super::virtio::Blockdev;

pub struct Arfs {
  dev: Blockdev
}

impl Arfs {
  pub fn new(dev: Blockdev) -> Self {
    Arfs { dev: dev }
  }

  pub fn read(&mut self, name: &str, buf: &mut[u8]) -> Result<usize, ()> {
    Err(())
  }
}
