// We can now talk to the actual virtio device
// via the CPU's I/O pins directly. A couple of helpful references:
//
// http://ozlabs.org/~rusty/virtio-spec/virtio-0.9.5.pdf
//     This is the actual virtio spec.
//
// http://ozlabs.org/~rusty/virtio-spec/virtio-paper.pdf
//     This is an academic paper describing the virtio design and architecture,
//     and how a virtqueue works and is implemented.
//
// https://www.freebsd.org/cgi/man.cgi?query=virtio&sektion=4
//     This is actually a FreeBSD manpage that gives a pretty good high-
//     level overview of how the guest kernel usually interacts with the
//     virtio interfaces and how it presents them to the guest OS's file
//     system.


// TODO(perf): the Tag could probably just be
mod virtq;
mod vring;
pub mod serial;

use prelude::*;

use cpuio;
use sched;
use sched::blocking::{WaitToken,SignalToken};
use block::{Client,Error};

extern {
  fn asm_eoi();
}

unsafe impl Sync for Blockdev {} // TODO

use core::cell::RefCell;
use collections::btree_map::BTreeMap;

type SignalMap = BTreeMap<u16,SignalToken>;

#[derive(Debug)]
pub struct Blockdev {
  port: RefCell<cpuio::IoPort>, // exclusive access to the entire port, for now

  q: GlobalMutex<virtq::Virtq>,

  completed_requests: Arc<GlobalMutex<BTreeMap<u16,virtq::Buf>>>,
}

use sync::global_mutex::GlobalMutex;
use byteorder::{ByteOrder,NativeEndian};

impl Client for Blockdev {
  type Tag = u16;

  fn read_dispatch(&self, sector: u64, mut buf: Box<[u8]>) -> Result<Self::Tag, Error> {
    assert_eq!(512, buf.len());

    println!("virtio blockdev: Reading sector {}", sector);
    let mut port = self.port.borrow_mut();

    let mut hdr = box [0u8; 16];
    NativeEndian::write_u32(&mut hdr[0..], 0); // kind: 0=read
    NativeEndian::write_u32(&mut hdr[4..], 1); // ioprio
    NativeEndian::write_u64(&mut hdr[8..], sector as u64);
    let mut done = box [17u8; 1]; // != 0 for checking that it was set by the host

    // 1. get id
    let mut q = self.q.lock();
    let id = q.register_and_send_rww(hdr, buf, done);

    // 3. use id
    q.xx(id, &mut *port);

    // println!("Now in flight, kicking: {:?}", self.in_flight);

    // // TODO: don't kick unconditionally
    // port.write16(16, 0);

    Ok(id as Self::Tag)
  }

  fn read_await(&self, tag: Self::Tag) -> Result<Box<[u8]>, Error> {
    // FIXME: make sure that we're not holding any borrows or locks before we go to sleep?
    // TODO loopy
    let condvar = self.q.lock().device_activity.clone();
    condvar.wait();

    match self.completed_requests.lock().remove(&tag).unwrap() {
      // drop hdr
      virtq::Buf::Rww(id1, _, id2, data, id3, done) => {
        {
          let mut d = self.q.lock();
          d.free_descriptors.push_back(id1);
          d.free_descriptors.push_back(id2);
          d.free_descriptors.push_back(id3);
        }

        println!("Virtio call completed, retval={}", done[0]);
        if done[0] != 0 { // retval of 0 indicates success
          println!("Virtio retval is {} != 0", done[0]);
          return Err(Error::InternalError)
        }
        return Ok(data)
      }
      _ => { panic!("unexpected buffer type") }
    }

    Err(Error::InternalError)
  }
}



#[derive(Debug)]
pub enum InitError {
  VirtioHandshakeFailure,
  NoDiskMarker,
  VirtioRequestFailed,
}

const VIRTIO_STATUS_ACKNOWLEDGE: u8 = 1;
const VIRTIO_STATUS_DRIVER: u8 = 2;
const VIRTIO_STATUS_DRIVER_OK: u8 = 4;
const VIRTIO_STATUS_FAILED: u8 = 128;

impl Blockdev {
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

    // Feature negotiation
    let offered_featureflags = configport.read16(0);
    println!("The device offered us these feature bits: {:?}", offered_featureflags);
    // In theory, we'd do `negotiated = offered & supported`; we don't actually
    // support any flags, so we can just set 0.
    configport.write16(4, 0);
    if configport.read16(4) != 0 {
      return Err(InitError::VirtioHandshakeFailure)
    }

    // The tokens for tagged I/O that allow the IRQ handler to wake up the specific
    // task that is waiting for the finished I/O.
    let completed = Arc::new(GlobalMutex::new(BTreeMap::new()));
    let completed_irqside = completed.clone();

    let (mut q, rx) = virtq::Virtq::new(0, &mut txport, box move |used, free| {
      let ref mut completed = completed_irqside.lock();

      for (buf, written) in used.drain(..) {
        assert_eq!(513, written);
        match buf {
          virtq::Buf::Rww(id, hdr, datadesc, data, donedesc, done) => {
            println!("Request with tag {} is completed", id);
            assert!(completed.insert(id, virtq::Buf::Rww(id, hdr, datadesc, data, donedesc, done)).is_none());
          }
          _ => {
            panic!("unexpected buffer type");
          }
        }
      }
    });

    let handler = virtq::RxHandler {
      rings: vec![rx],
      isr_status_port: rxport,
    };

    sched::irq::add_handler(0x2a, box handler);

    // Tell the device we're done setting it up
    state = state | VIRTIO_STATUS_DRIVER_OK;
    configport.write8(18, state);

    Ok(Blockdev {
      port: RefCell::new(txport),
      q: GlobalMutex::new(q),
      completed_requests: completed,
    })
  }
}
