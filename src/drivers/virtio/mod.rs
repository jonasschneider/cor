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


mod queue;
mod vring;
pub mod serial;

use prelude::*;

use cpuio;
use sched;
use sched::blocking::{WaitToken,SignalToken};
use block::{Client,ReadWaitToken,Error};

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
  pool: RefCell<queue::BufferPool>,

  in_flight: GlobalMutex<BTreeMap<ReadWaitToken,(Box<[u8]>,Box<[u8]>,WaitToken)>>,
  wakeup_tokens: Arc<GlobalMutex<SignalMap>>,
}


// Handles receive notifications for a virtio device.
#[derive(Debug)]
struct RxHandler {
  isr_status_port: cpuio::IoPort,

  // The rings to receive on, and who to notify on receive
  rings: Vec<(vring::Used,Arc<GlobalMutex<SignalMap>>)>,
}

impl sched::irq::InterruptHandler for RxHandler {
  fn critical(&mut self) {
    // The virtio IRQ status is reset by **reading** from this port
    if self.isr_status_port.read8(19) & 1 == 0  {
      println!("ISR==0, this interrupt likely wasn't for us.");
      return;
    }

    for &mut(ref mut used, ref signals_lk) in &mut self.rings {
      let ref mut signals = signals_lk.lock();
      while let Some(ref descid) = used.take_from_ring() {
        let wakeup = signals.remove(&descid).unwrap(); // remove from the set of in-flight reqs, panic if absent
        println!("Buffer {} is used, signalling {:?}", descid, wakeup);
        wakeup.signal();
      }
    }

    unsafe { asm_eoi(); } // TODO
  }

  fn noncritical(&self) {
    println!("NONcritical self: {:?}", self);
  }
}

use sync::global_mutex::GlobalMutex;
use byteorder::{ByteOrder,NativeEndian};

impl Client for Blockdev {
  fn read_dispatch(&self, sector: u64, mut buf: Box<[u8]>) -> Result<ReadWaitToken, Error> {
    println!("virtio blockdev: Reading sector {}", sector);
    let mut pool = self.pool.borrow_mut();
    let mut port = self.port.borrow_mut();

    assert_eq!(512, buf.len());

    let mut hdr = [0u8; 16];
    NativeEndian::write_u32(&mut hdr[0..], 0); // kind: 0=read
    NativeEndian::write_u32(&mut hdr[4..], 1); // ioprio
    NativeEndian::write_u64(&mut hdr[8..], sector as u64);
    let mut done = box [17u8; 1]; // != 0 for checking that it was set by the host

    // todo: need to think harder about wraparound and wait token uniqueness
    // todo: this is actually a race, we need to set the descriptor, but *not* enqueue yet
    //       before adding to wakeup_tokens
    let descriptor_i = pool.enqueue_rww(&hdr, &mut buf[..], &mut done[..]).unwrap();
    let tok = descriptor_i as ReadWaitToken;
    println!("enqueued it: {:?}", &pool.avail.mem[128*16..]);

    let mut n = String::new();
    write!(n, "virtio-block read of sector {}", sector);
    let (wait, signal) = sched::blocking::tokens(n);
    self.in_flight.lock().insert(tok, (done, buf, wait));

    self.wakeup_tokens.lock().insert(descriptor_i, signal);

    println!("Now in flight, kicking: {:?}", self.in_flight);

    // TODO: don't kick unconditionally
    port.write16(16, 0);

    Ok(tok)
  }

  fn read_await(&self, tok: ReadWaitToken) -> Result<Box<[u8]>, Error> {
    let mut port = self.port.borrow_mut();

    let (done, buf, wait_tok) = self.in_flight.lock().remove(&tok).unwrap(); // panics on invalid token / double read?

    // FIXME: make sure that we're not holding any of these borrows or locks before we go to sleep
    wait_tok.wait();
    println!("wait token signals completion of: {:?}", tok);
    println!("Virtio call completed, retval={}", done[0]);

    if done[0] != 0 { // retval of 0 indicates success
      println!("Virtio retval is {} != 0", done[0]);
      return Err(Error::InternalError)
    }

    Ok(buf)
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

    // Now comes the block-device-specific setup.
    // (The configuration of a single virtqueue isn't device-specific though; it's the same
    // for i.e. the virtio network controller)

    // Discover virtqueues; the block devices only has one
    if configport.read16(4) != 0 {
      return Err(InitError::VirtioHandshakeFailure)
    }
    // initialize the first (and only, for block devices) virtqueue

    let (mut pool, mut used) = queue::setup(0, &mut txport);
    // Tell the device we're done setting it up
    state = state | VIRTIO_STATUS_DRIVER_OK;
    configport.write8(18, state);

    println!("Device state is now: {}", state);

    // if q.test(&port) {
    //   println!("Virtio-blk device successfully initialized and tested!");
    // } else {
    //   panic!("Self-test failed!")
    // }

    let wakeup = Arc::new(GlobalMutex::new(BTreeMap::new()));

    let handler = RxHandler {
      rings: vec![(used,wakeup.clone())],
      isr_status_port: rxport,
    };

    sched::irq::add_handler(0x2a, box handler);

    Ok(Blockdev { pool: RefCell::new(pool), port: RefCell::new(txport),
      in_flight: GlobalMutex::new(BTreeMap::new()),
      wakeup_tokens: wakeup
       })
  }
}
