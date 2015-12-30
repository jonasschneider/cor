use prelude::*;

use core::borrow::BorrowMut;

use cpuio;
use super::{vring,InitError};
const VIRTIO_STATUS_ACKNOWLEDGE: u8 = 1;
const VIRTIO_STATUS_DRIVER: u8 = 2;
const VIRTIO_STATUS_DRIVER_OK: u8 = 4;
const VIRTIO_STATUS_FAILED: u8 = 128;

const VRING_DESC_F_NEXT: u16 = 1; /* This marks a buffer as continuing via the next field. */
const VRING_DESC_F_WRITE: u16 = 2; /* This marks a buffer as write-only (otherwise read-only). */

use super::vring::Descriptor;
use mem::*;

use sched;

extern {
  fn asm_eoi();
}

pub struct Serialdev {
  port: cpuio::IoPort,
  rxbuf: Box<[u8; 20]>,

  rxavail: vring::Avail,
  rxused: vring::Used,

  txq: super::virtq::Virtq,
}

impl Serialdev {
  pub fn putc(&mut self, c: char) {
    let mut b = [0u8; 1];
    b[0] = c as u8;
    let n = self.txq.send(&b[..], &mut self.port);

    println!("serial send done");
  }

  pub fn read(&mut self, buf: &mut[u8]) -> usize {
    // self.next = self.next + 1; // just for safety, i think descriptors might be cached?

    // self.rxavail.write_descriptor_at(self.next as usize, Descriptor {
    //   addr: physical_from_kernel((buf[..]).as_ptr() as usize) as u64,
    //   len: buf.len() as u32,
    //   flags: VRING_DESC_F_WRITE,
    //   next: 0,
    // });
    // self.rxavail.add_to_ring(self.next);

    println!("before wait: {:?}", self.rxbuf);

    self.rxavail.add_to_ring(0);
    self.port.write16(16, 0);

    let mut n;
    loop {
      match self.rxused.take_from_ring() {
        None => {},
        Some((ref bufdesc, ref read)) => { n=*read;break; }
      }
    }

    buf.clone_from_slice(&self.rxbuf[0..n]);

    println!("after wait: {:?}", self.rxbuf);

    n
  }

  pub fn new(mut port: cpuio::IoPort) -> Result<Self, InitError> {
    println!("Initializing virtio block device with ioport {:?}..", port);

    let (mut configport, mut operationsport) = port.split_at_masks(
      "XXXXXXXX----------X-XXXX",  // feature negotiation flags, device status, MSI-X fields
      "--------XXXXXXXXXX-X----"); // queue address, size, select, notify, ISR status

    let (mut rxport, mut txport) = operationsport.split_at_masks(
      "-------------------X----",  // ISR status
      "--------XXXXXXXXXX------"); // queue address, size, select, notify

    let mut state = 0u8;
    configport.write8(18, state);

    state = state | VIRTIO_STATUS_ACKNOWLEDGE;
    configport.write8(18, state);

    state = state | VIRTIO_STATUS_DRIVER;
    configport.write8(18, state);

    // Feature negotiation; TODO: the feature registers are actually 32-bit wide
    let offered_featureflags = configport.read16(0);
    println!("The device offered us these feature bits: {:?}", offered_featureflags);
    // In theory, we'd do `negotiated = offered & supported`; we don't actually
    // support any flags, so we can just set 0.
    configport.write16(4, 0); // XX


    // Now comes the block-device-specific setup.
    // (The configuration of a single virtqueue isn't device-specific though; it's the same
    // for i.e. the virtio network controller)

    // Discover virtqueues; the block devices only has one
    if configport.read16(4) != 0 {
      return Err(InitError::VirtioHandshakeFailure)
    }


    // RX

        // Set queue_select
        txport.write16(14, 0);

        // Determine how many descriptors the queue has, and allocate memory for the
        // descriptor table and the ring arrays.
        let rxlength = txport.read16(12);
        println!("Max rx len is: {}",rxlength);
        let (rxaddress, mut rxavail, mut rxused) = vring::setup(rxlength);

        let rxp = physical_from_kernel(rxaddress as usize) as u32; // FIXME: not really a safe cast
        txport.write32(8, rxp >> 12);

        let rxbuf = box ['X' as u8; 20];
        rxavail.write_descriptor_at(0, Descriptor {
          addr: physical_from_kernel((rxbuf[..]).as_ptr() as usize) as u64,
          len: 20,
          flags: VRING_DESC_F_WRITE,
          next: 0,
        });
        rxavail.add_to_ring(0);

    // ctrl

        txport.write16(14, 2); // CTRL-RX

        let ctrllength = txport.read16(12);
        println!("Ctrl qsz is: {}",ctrllength);
        let (ctrladdress, mut ctrlavail, mut ctrlused) = vring::setup(ctrllength);

        let ctrlp = physical_from_kernel(ctrladdress as usize) as u32; // FIXME: not really a safe cast
        txport.write32(8, ctrlp >> 12);

        let cbuf = box [0u8; 8];
        ctrlavail.write_descriptor_at(0, Descriptor {
          addr: physical_from_kernel((cbuf[..]).as_ptr() as usize) as u64,
          len: 8,
          flags: VRING_DESC_F_WRITE,
          next: 0,
        });
        ctrlavail.add_to_ring(0);

    // TX

    let (mut txq, txrecv) = super::virtq::Virtq::new(1, &mut txport);

    println!("txq: {:?}", &txq);

    let handler = super::virtq::RxHandler {
      rings: vec![txrecv],
      isr_status_port: rxport,
    };

    sched::irq::add_handler(0x2b, box handler);





    // // Set queue_select
    // txport.write16(14, 1);

    // // Determine how many descriptors the queue has, and allocate memory for the
    // // descriptor table and the ring arrays.
    // let length = txport.read16(12);
    // println!("Max tx len is: {}",rxlength);
    // let (address, mut avail, mut used) = vring::setup(length);

    // let physical_32 = physical_from_kernel(address as usize) as u32; // FIXME: not really a safe cast
    // txport.write32(8, physical_32 >> 12);

    // println!("Device state is now: {}", state);

    // for _ in 0..10 {
    //     register

    for _ in 0..100 {
      txq.register(box ['X' as u8; 1], false);
    }

    println!("txq with bufs: {:?}", &txq);

    // for i in 0..(length as usize) {
    //   let buf = box ['X' as u8; 1];
    //   let data: *const u8 = buf.as_ptr();
    //   avail.write_descriptor_at(i, Descriptor {
    //     addr: physical_from_kernel(data as usize) as u64,
    //     len: 1,
    //     flags: 0,
    //     next: 0,
    //   });
    //   bufs.push(buf);
    // }
    // avail.add_to_ring(0);
    // avail.add_to_ring(1);

    // let mut buf = box ['X' as u8; 100];
    // buf[20] = '\n' as u8;
    // avail.write_descriptor_at(0, Descriptor {
    //   addr: physical_from_kernel(buf.as_ptr() as usize) as u64,
    //   len: 1,
    //   flags: 0,
    //   next: 0,
    // });
    // bufs.push(buf);
    //avail.add_to_ring(0);

    // Tell the device we're done setting it up
    state = state | VIRTIO_STATUS_DRIVER_OK;
    configport.write8(18, state);
    println!("Device state is now: {}", state);
    // println!("ISR: {}", rxport.read8(19));

    // println!("rxbuf: {:?}", rxbuf);
    // println!("Triggering..");

    // // notify on both queues
    // txport.write16(16, 1);
    // txport.write16(16, 0);
    // //txport.write16(16, 2);


    // // let handler = RxHandler {
    // //   rings: vec![(used,wakeup.clone())],
    // //   isr_status_port: rxport,
    // // };

    // // a PORT_OPEN(0) ctrl message (indicating the port is closed) is sent after EOF on stdin, i.e. if you pipe something into qemu

    // //sched::irq::add_handler(0x2b, box handler);

    // println!("tx: {:?}", used.take_from_ring());
    // println!("rx: {:?}", rxused.take_from_ring());

    // println!("ctrl-rx: {:?}", ctrlused.take_from_ring());
    // println!("ISR: {}", rxport.read8(19));
    // unsafe { asm_eoi(); }
    // println!("rxbuf: {:?}", rxbuf);

    // avail.add_to_ring(0);

    // txport.write16(16, 1);
    // //txport.write16(16, 0);
    // //txport.write16(16, 2);


    // println!("Waiting for take() on rx-used..");
    // while let None = rxused.take_from_ring() {}
    // println!("done");

    // println!("tx: {:?}", used.take_from_ring());
    // println!("rx: {:?}", rxused.take_from_ring());
    // println!("ctrl-rx: {:?} ({:?})", ctrlused.take_from_ring(), cbuf);
    // println!("ISR: {}", rxport.read8(19));
    // unsafe { asm_eoi(); }
    // println!("rxbuf: {:?}", rxbuf);

    // //panic!("done");

    let mut dev = Serialdev { port: txport, rxbuf: rxbuf,
      rxavail: rxavail, rxused: rxused, txq: txq};

    dev.putc('?');
    dev.putc('!');

    Ok(dev)
  }
}
