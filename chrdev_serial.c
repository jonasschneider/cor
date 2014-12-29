#include "io.h"

#define PORT 0x3f8   /* COM1 */

void cor_chrdev_serial_init() {
   cor_outb(0x00, PORT + 1);    // Disable all interrupts
   cor_outb(0x80, PORT + 3);    // Enable DLAB (set baud rate divisor)
   cor_outb(0x03, PORT + 0);    // Set divisor to 3 (lo byte) 38400 baud
   cor_outb(0x00, PORT + 1);    //                  (hi byte)
   cor_outb(0x03, PORT + 3);    // 8 bits, no parity, one stop bit
   cor_outb(0xC7, PORT + 2);    // Enable FIFO, clear them, with 14-byte threshold
   cor_outb(0x0B, PORT + 4);    // IRQs enabled, RTS/DSR set
}
int serial_received() {
   return cor_inb(PORT + 5) & 1;
}
char cor_chrdev_serial_read() {
   while (serial_received() == 0);

   return cor_inb(PORT);
}
int is_transmit_empty() {
   return cor_inb(PORT + 5) & 0x20;
}
void cor_chrdev_serial_write(char a) {
   while (is_transmit_empty() == 0);
   cor_outb(a, PORT);
}
