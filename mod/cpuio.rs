pub type Port = u16;

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

pub unsafe fn putc(c : char) {
  write8(0x3f8, c as u8)
}
