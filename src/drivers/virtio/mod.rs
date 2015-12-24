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

use cpuio;

use kalloc::__rust_allocate as allocate;
use alloc::boxed::Box;
use core;
use core::slice;
use core::fmt;
use kbuf;
use collections;
use collections::vec::Vec;
use sched;
use block::{Client,ReadWaitToken,Error};

extern {
  fn asm_eoi();
}

unsafe impl Sync for Blockdev {} // TODO

use core::cell::RefCell;
use collections::btree_map::BTreeMap;

#[derive(Debug)]
pub struct Blockdev {
  port: RefCell<cpuio::IoPort>, // exclusive access to the entire port, for now
  pool: RefCell<queue::BufferPool>,
  used: RefCell<vring::Used>,

  in_flight: GlobalMutex<BTreeMap<ReadWaitToken,(Box<[u8]>,Box<[u8]>)>>,
}


// // Handles IRQs for a specific virtio device.
// // (Multiple devices need multiple handlers.)
// struct IRQRx {
//   isr_status_port: cpuio::IoPort,

//   // The rings to receive on
//   rings: Vec<VringUsed>,
// }

// impl sched::irq::Handler for IRQRx {
// }
use sync::global_mutex::GlobalMutex;
use core::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
static NEXT_TOKEN: AtomicUsize = ATOMIC_USIZE_INIT;

use byteorder::{ByteOrder,NativeEndian};

impl Client for Blockdev {
  fn read_dispatch(&self, sector: u64, mut buf: Box<[u8]>) -> Result<ReadWaitToken, Error> {
    let mut pool = self.pool.borrow_mut();
    let mut port = self.port.borrow_mut();

    assert_eq!(512, buf.len());

    let mut hdr = [0u8; 16];
    NativeEndian::write_u32(&mut hdr[0..], 0); // kind: 0=read
    NativeEndian::write_u32(&mut hdr[4..], 1); // ioprio
    NativeEndian::write_u64(&mut hdr[8..], sector as u64);
    let mut done = box [17u8; 1]; // != 0 for checking that it was set by the host

    pool.enqueue_rww(&hdr, &mut buf[..], &mut done[..]);

    println!("enqueued it: {:?}", &pool.avail.mem[128*16..]);

    // always kick here, TODO: chaining for performance
    port.write16(16, 0);

    let tok = NEXT_TOKEN.fetch_add(1, Ordering::SeqCst) as ReadWaitToken;
    self.in_flight.lock().insert(tok, (done,buf));

    println!("Now in flight: {:?}", self.in_flight);
    Ok(tok)
  }

  fn read_await(&self, tok: ReadWaitToken) -> Result<Box<[u8]>, Error> {
    let mut port = self.port.borrow_mut();
    let mut used = self.used.borrow_mut();

    let (done, buf) = self.in_flight.lock().remove(&tok).unwrap(); // panics on invalid token / double read?

    while let None = used.take_from_ring() {
      port.write16(16, 0);
      sched::park_until_irq(0x2b);
      println!("After park: {:?}", used);
    }

    // TODO: this whole IRQ handling is really bad.
    // Especially, I feel like we should never call asm_eoi from Rustland
    // (or from non-interrupt kernel land entirely).
    // Mark the virtio irq as handled *before* EOI, otherwise we'd get another one right away.
    port.read8(19); // The virtio IRQ status is reset by **reading** from this port
    unsafe { asm_eoi(); }

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

    let (mut pool, mut used) = queue::setup(0, &mut operationsport);
    // Tell the device we're done setting it up
    state = state | VIRTIO_STATUS_DRIVER_OK;
    configport.write8(18, state);

    println!("Device state is now: {}", state);

    // if q.test(&port) {
    //   println!("Virtio-blk device successfully initialized and tested!");
    // } else {
    //   panic!("Self-test failed!")
    // }

    Ok(Blockdev { pool: RefCell::new(pool), used: RefCell::new(used), port: RefCell::new(operationsport),
      in_flight: GlobalMutex::new(BTreeMap::new()) })
  }
}
