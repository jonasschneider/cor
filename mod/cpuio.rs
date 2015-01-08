type Port = u16;

pub unsafe fn write8(port: Port, value : u8) {
  asm! (
    "outb %al, %dx"
    :
    : "{dx}" (port as u16), "{al}" (value)
    :
    : "volatile"
  );
}

pub unsafe fn putc(c : char) {
  write8(0x3f8, c as u8)
}
