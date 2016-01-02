use prelude::*;

use cpuio;
use super::{virtq,InitError};

const VIRTIO_STATUS_ACKNOWLEDGE: u8 = 1;
const VIRTIO_STATUS_DRIVER: u8 = 2;
const VIRTIO_STATUS_DRIVER_OK: u8 = 4;
const VIRTIO_STATUS_FAILED: u8 = 128;

use mem::*;

use sched;

pub struct Serialdev {
  port: cpuio::IoPort,

  rxq: virtq::Virtq,
  txq: virtq::Virtq,
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

      if let Some((b, count)) = r {
        match b {
          virtq::Buf::Simple(desc, data) => {
            n = count;

            buf.clone_from_slice(&data[0..n]);

            // enqueue the buffer again for the next read
            self.rxq.free_buffers.lock().push_back(virtq::Buf::Simple(desc, data));
            self.rxq.send(&[0u8; 20], &mut self.port);

            break;
          },
          _ => { panic!("unexpected buffer type"); }
        }
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

    let (mut irqport, mut userport) = operationsport.split_at_masks(
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
    configport.write16(4, 0);
    // TODO: handle featureflags; a PORT_OPEN(0) ctrl message (indicating the port was closed)
    // is sent after EOF on qemu's stdin if the multiport feature was negotiated
    if configport.read16(4) != 0 {
      return Err(InitError::VirtioHandshakeFailure)
    }

    // Set up the rx & tx virtqueues.

    // Rx Behaviour: Move stuff into an "unread queue" -> TODO: encapsulation
    let (mut rxq, rxrecv) = virtq::Virtq::new(0, &mut userport, box move |used, free| {
      println!("serialrx processing used buffers: {:?}", used);
    });
    for _ in 0..1 {
      rxq.register(box ['X' as u8; 20], true); // writable by them
      rxq.send(&[0u8; 20], &mut userport);
    }

    // Tx Behaviour: When the device consumes a tx buffer, we simply re-queue
    // it as a free buffer.
    let (mut txq, txrecv) = virtq::Virtq::new(1, &mut userport, box move |used, free| {
      use core::iter::Extend;

      println!("serialtx processing used buffers: {:?}", used);
      free.lock().extend(used.drain(..).map(|(buf, read)| buf));
    });
    for _ in 0..10 {
      txq.register(box ['X' as u8; 1], false); // not writable by them
    }

    let handler = virtq::RxHandler {
      rings: vec![rxrecv, txrecv],
      isr_status_port: irqport,
    };

    sched::irq::add_handler(0x2b, box handler);

    state = state | VIRTIO_STATUS_DRIVER_OK;
    configport.write8(18, state);

    Ok(Serialdev { port: userport, rxq: rxq, txq: txq })
  }
}
