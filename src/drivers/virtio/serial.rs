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

        // txport.write16(14, 2); // CTRL-RX

        // let ctrllength = txport.read16(12);
        // println!("Ctrl qsz is: {}",ctrllength);
        // let (ctrladdress, mut ctrlavail, mut ctrlused) = vring::setup(ctrllength);

        // let ctrlp = physical_from_kernel(ctrladdress as usize) as u32; // FIXME: not really a safe cast
        // txport.write32(8, ctrlp >> 12);

        // let cbuf = box [0u8; 8];
        // ctrlavail.write_descriptor_at(0, Descriptor {
        //   addr: physical_from_kernel((cbuf[..]).as_ptr() as usize) as u64,
        //   len: 8,
        //   flags: VRING_DESC_F_WRITE,
        //   next: 0,
        // });
        // ctrlavail.add_to_ring(0);

    // TX

    // Behaviour: When the device consumes a tx buffer, we simply re-queue
    // it as a free buffer.
    let (mut txq, txrecv) = super::virtq::Virtq::new(1, &mut txport, box move |used, free| {
      use core::iter::Extend;

      println!("serialtx processing used buffers: {:?}", used);
      free.lock().extend(used.drain(..));
    });

    println!("txq: {:?}", &txq);

    let handler = super::virtq::RxHandler {
      rings: vec![txrecv],
      isr_status_port: rxport,
    };

    sched::irq::add_handler(0x2b, box handler);

    for _ in 0..10 {
      txq.register(box ['X' as u8; 1], false);
    }

    // Tell the device we're done setting it up
    state = state | VIRTIO_STATUS_DRIVER_OK;
    configport.write8(18, state);
    println!("Device state is now: {}", state);

    // TODO: a PORT_OPEN(0) ctrl message (indicating the port was closed) is sent after EOF on qemu's stdin

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
