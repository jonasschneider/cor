use prelude::*;

use cpuio;
use super::{virtq,InitError};
use super::pci;
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
    let rxhandler = (box move |used, free| {
      println!("serialrx processing used buffers: {:?}", used);
    }) as virtq::Handler;

    // Tx Behaviour: When the device consumes a tx buffer, we simply re-queue
    // it as a free buffer.
    let txhandler = (box move |used, free| {
      use core::iter::Extend;

      println!("serialtx processing used buffers: {:?}", used);
      free.lock().extend(used.drain(..).map(|(buf, read)| buf));
    }) as virtq::Handler;

    let handlers = vec![(0, rxhandler), (1, txhandler)];
    let (mut qs, mut txport) = pci::init(port, 0x2b, handlers);

    let mut txq = qs.remove(1);
    let mut rxq = qs.remove(0);

    for _ in 0..1 {
      rxq.register(box ['X' as u8; 20], true); // writable by them
      rxq.send(&[0u8; 20], &mut txport);
    }

    for _ in 0..10 {
      txq.register(box ['X' as u8; 1], false); // not writable by them
    }

    Ok(Serialdev { port: txport, rxq: rxq, txq: txq })
  }
}
