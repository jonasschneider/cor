// TODO: dynamically ensure that there are never two IoPorts for overlapping regions
// static mut alloc : Option<PortAllocator> = None;
// struct PortAllocator {
// }

#[derive(Debug)]
pub struct IoPort {
  base: u16,
  pub width: u16,
}

pub fn alloc(base: u16, width: u16) -> Result<IoPort, ()> {
  Ok(IoPort{ base: base, width: width })
}

impl IoPort {
  pub fn write8(&self, offset: u16, val: u8) {
    assert!(offset < self.width);
    unsafe { write8(self.base+offset, val) }
  }
  pub fn write16(&self, offset: u16, val: u16) {
    assert!(offset+1 < self.width);
    unsafe { write16(self.base+offset, val) }
  }
  pub fn write32(&self, offset: u16, val: u32) {
    assert!(offset+3 < self.width);
    unsafe { write32(self.base+offset, val) }
  }

  // TODO: check limit as well
  pub fn read8(&self, offset: u16) -> u8 { unsafe { read8(self.base+offset) } }
  pub fn read16(&self, offset: u16) -> u16 { unsafe { read16(self.base+offset) } }
  //n read32(&self, offset: u16) { unsafe { read32(self.base+offset) } }
}

pub type Port = u16;

pub unsafe fn write32(port: Port, value : u32) {
  asm! (
    "outl %eax, %dx"
    :
    : "{dx}" (port as u16), "{eax}" (value)
    :
    : "volatile"
  );
}

pub unsafe fn write8(port: Port, value : u8) {
  asm! (
    "outb %al, %dx"
    :
    : "{dx}" (port as u16), "{al}" (value)
    :
    : "volatile"
  );
}

pub unsafe fn write16(port: Port, value : u16) {
  asm! (
    "outw %ax, %dx"
    :
    : "{dx}" (port as u16), "{ax}" (value)
    :
    : "volatile"
  );
}

pub unsafe fn read16(port: Port) -> u16 {
  let mut x : u16 = 0;
  asm! (
    "inw %dx, %ax"
    : "={ax}" (x)
    : "{dx}" (port as u16)
    :
    : "volatile"
  );
  x
}

pub unsafe fn read8(port: Port) -> u8 {
  let mut x : u8 = 0;
  asm! (
    "inb %dx, %al"
    : "={al}" (x)
    : "{dx}" (port as u16)
    :
    : "volatile"
  );
  x
}

pub unsafe fn putc(c : char) {
  write8(0x3f8, c as u8)
}
