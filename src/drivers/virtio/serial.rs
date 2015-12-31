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

  rxq: super::virtq::Virtq,
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
    let mut n;
    loop {
      // Make sure that we don't keep the lock held when we possibly call kyield()
      // TODO: Enforce this using the type systems (sleep tokens that downgrade to spinlock tokens)
      let r = {
        let mut lock = self.rxq.used_buffers.lock();
        lock.pop_front()
      };

      if let Some((rxbuf, read)) = r {
        n = read;

        buf.clone_from_slice(&rxbuf.1[0..n]);
        println!("read: {:?}", &rxbuf);

        // enqueue the buffer again for the next read
        self.rxq.free_buffers.lock().push_back(rxbuf);
        self.rxq.send(&[0u8; 20], &mut self.port);

        break;
      }
      sched::kyield();
    }

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

    // Discover virtqueues; the block devices only has one
    if configport.read16(4) != 0 {
      return Err(InitError::VirtioHandshakeFailure)
    }

    // RX

    // Behaviour: Move stuff into an "unread queue" -> TODO: encapsulation
    let (mut rxq, rxrecv) = super::virtq::Virtq::new(0, &mut txport, box move |used, free| {
      println!("serialrx processing used buffers: {:?}", used);
    });
    for _ in 0..1 {
      rxq.register(box ['X' as u8; 20], true); // writable by them
      rxq.send(&[0u8; 20], &mut txport);
    }

    // TX

    // Behaviour: When the device consumes a tx buffer, we simply re-queue
    // it as a free buffer.
    let (mut txq, txrecv) = super::virtq::Virtq::new(1, &mut txport, box move |used, free| {
      use core::iter::Extend;

      println!("serialtx processing used buffers: {:?}", used);
      free.lock().extend(used.drain(..).map(|(buf, read)| buf));
    });
    for _ in 0..10 {
      txq.register(box ['X' as u8; 1], false);
    }

    let handler = super::virtq::RxHandler {
      rings: vec![rxrecv, txrecv],
      isr_status_port: rxport,
    };

    sched::irq::add_handler(0x2b, box handler);


    // Tell the device we're done setting it up
    state = state | VIRTIO_STATUS_DRIVER_OK;
    configport.write8(18, state);
    println!("Device state is now: {}", state);

    // TODO: a PORT_OPEN(0) ctrl message (indicating the port was closed) is sent after EOF on qemu's stdin

    let mut dev = Serialdev { port: txport, rxq: rxq, txq: txq};

    dev.putc('?');
    dev.putc('!');

    Ok(dev)
  }
}
