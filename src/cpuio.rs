// TODO: dynamically ensure that there are never two IoPorts for overlapping regions
// static mut alloc : Option<PortAllocator> = None;
// struct PortAllocator {
// }

type Address = u16;

#[derive(Debug)]
pub struct IoPort {
  base: u16,
  pub width: u16,
  mask: &'static str,
}

// Unsafe because you could call it multiple times.
pub unsafe fn alloc(base: u16, width: u16, mask: &'static str) -> Result<IoPort, ()> {
  debug_assert_eq!(width as usize, mask.len());
  Ok(IoPort{ base: base, width: width, mask: mask })
}

// these should probably just be debug_assert!'s
impl IoPort {
  // Given two byte masks containing 'X' and '-', split the given Port into two
  // non-overlapping masked ports.
  // Panics if the given masks are illegal.
  pub fn split_at_masks(self, newmask_a: &'static str, newmask_b: &'static str) -> (Self, Self) {
    debug_assert_eq!(self.width as usize, newmask_a.len());
    debug_assert_eq!(self.width as usize, newmask_b.len());

    // Such zipping, much higher-order functions, wow!
    for (cur, a, b) in self.mask.chars().zip(newmask_a.chars()).zip(newmask_b.chars()).map(|((x,y), z)| (x,y,z)) {
      if (cur == '-') && (a != '-' || b != '-') {
        panic!("New mask contains entries that were already masked out! A: {}, B: {}, present: {}", newmask_a, newmask_b, self.mask);
      }
      if a != '-' && b != '-' {
        panic!("New masks overlap! A: {}, B: {}, present: {}", newmask_a, newmask_b, self.mask);
      }
    }

    (IoPort{ base: self.base, width: self.width, mask: newmask_a },
     IoPort{ base: self.base, width: self.width, mask: newmask_b })
  }

  pub fn write8(&mut self, offset: u16, val: u8) {
    assert!(offset < self.width);
    assert_eq!(self.mask.char_at((offset)   as usize), 'X');
    unsafe { write8(self.base+offset, val) }
  }
  pub fn write16(&mut self, offset: u16, val: u16) {
    assert!(offset+1 < self.width);
    assert_eq!(self.mask.char_at((offset)   as usize), 'X');
    assert_eq!(self.mask.char_at((offset+1) as usize), 'X');
    unsafe { write16(self.base+offset, val) }
  }
  pub fn write32(&mut self, offset: u16, val: u32) {
    assert!(offset+3 < self.width);
    assert_eq!(self.mask.char_at((offset)   as usize), 'X');
    assert_eq!(self.mask.char_at((offset+1) as usize), 'X');
    assert_eq!(self.mask.char_at((offset+2) as usize), 'X');
    assert_eq!(self.mask.char_at((offset+3) as usize), 'X');
    unsafe { write32(self.base+offset, val) }
  }

  pub fn read8(&mut self, offset: u16) -> u8 {
    assert!(offset < self.width);
    assert_eq!(self.mask.char_at((offset)   as usize), 'X');
    unsafe { read8(self.base+offset) }
  }
  pub fn read16(&mut self, offset: u16) -> u16 {
    assert!(offset+1 < self.width);
    assert_eq!(self.mask.char_at((offset)   as usize), 'X');
    assert_eq!(self.mask.char_at((offset+1) as usize), 'X');
    unsafe { read16(self.base+offset) }
  }
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
