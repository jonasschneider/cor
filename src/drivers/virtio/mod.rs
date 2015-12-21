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

use block::{Client,ReadWaitToken,Error as Errorx};

extern {
  fn asm_eoi();
}

pub trait Block {
  // Read a block. Raises if `buf` does not have size 512.
  fn read(&mut self, sector: usize, buf: &mut [u8]) -> Result<(), Error>;
}

#[derive(Debug)]
pub struct Blockdev {
  port: cpuio::IoPort, // exclusive access to the entire port, for now
  q: queue::Virtqueue,
}


/*
(Sched idea: for sleeping/yielding, require passing around a yield token that represents
the permission to block the process. Would eliminate attempts to sleep from IRQ context
at compile time.)

pub fn init_virtio_block(ioport) -> (Box<BlockdevClient>, VirtioIRQHandler)
--> use Client in userspace, install IRQ handler

impl ::sched::irq::Handler for VirtioIRQHandler {
  ..
}

pub trait BlockdevClient: Sync {
  type ReadWaitToken;

  // Submits a sequest to read the specified sector.
  // Returns a token that can be used to block until the read is completed.
  fn read(&self, sector: usize) -> Result<ReadWaitToken, Error>;

  // Block until the read identified by the token is completed, then writes the read data
  // into `buf` (which must be of size 512).
  fn wait_read(&self, tok: ReadWaitToken, buf: &mut [u8]) -> Result<(), Error>;
}

// Handling the IRQs for a specific virtio device. (Multiple devices need multiple handlers.)
struct VirtioIRQHandler {
  isr_status_register: cpuio::IoPort,
  // The rings to receive on
  rings: Vec<VringUsed>,
}
*/


#[repr(C, packed)]
pub struct BlockRequest {
  kind: u32,
  ioprio: u32,
  sector: u64,
}

pub struct NewstyleClient(core::cell::UnsafeCell<Blockdev>);

unsafe impl Sync for NewstyleClient {} // TODO
impl Client for NewstyleClient {
  fn read(&self, sector: u64) -> Result<ReadWaitToken, Errorx> { Ok(sector) }
  fn wait_read(&self, tok: ReadWaitToken, buf: &mut [u8]) -> Result<(), Errorx> {
    let res = unsafe{(*self.0.get()).read(tok as usize, buf)};
    println!("Result: {:?}", res);
    Ok(())
  }
}

impl Block for Blockdev {
  fn read(&mut self, sector: usize, buf: &mut [u8]) -> Result<(), Error> {
    assert_eq!(512, buf.len());

    let mut hdr = BlockRequest {
      kind: 0, // 0=read
      ioprio: 1, // prio
      sector: sector as u64,
    };
    let mut done = [17u8; 1]; // != 0 for checking that it was set by the host

    {
      let hdrbuf: &[u8] = unsafe{ slice::from_raw_parts(core::mem::transmute(&hdr), core::mem::size_of::<BlockRequest>()) };
      self.q.ring.enqueue_rww(&hdrbuf, &mut buf[..], &mut done[..]);
    }

    // Finally, we "kick" the device to tell it that it should look for
    // something to do. We could probably skip doing this and just wait for a
    // while; even after a kick, there's no guarantee that the request will have
    // been processed. The actual notification about "I did a thing, please go
    // check" will be delivered back to us via an interrupt.
    // Now we park ourselves until things change.

    while let None = self.q.ring.take() {
      self.port.write16(16, 0);
      sched::park_until_irq(0x2b);
    }

    // TODO: this whole IRQ handling is really bad.
    // Especially, I feel like we should never call asm_eoi from Rustland
    // (or from non-interrupt kernel land entirely).
    // Mark the virtio irq as handled *before* EOI, otherwise we'd get another one right away.
    self.port.read8(19); // The virtio IRQ status is reset by **reading** from this port
    unsafe { asm_eoi(); }

    println!("Virtio call completed, retval={}", done[0]);

    if done[0] != 0 { // retval of 0 indicates success
      return Err(Error::VirtioRequestFailed)
    }

    Ok(())
  }
}



#[derive(Debug)]
pub enum Error {
  VirtioHandshakeFailure,
  NoDiskMarker,
  VirtioRequestFailed,
}

const VIRTIO_STATUS_ACKNOWLEDGE: u8 = 1;
const VIRTIO_STATUS_DRIVER: u8 = 2;
const VIRTIO_STATUS_DRIVER_OK: u8 = 4;
const VIRTIO_STATUS_FAILED: u8 = 128;


pub unsafe fn init(mut port: cpuio::IoPort) -> Result<Blockdev, Error> {
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

  let mut state = 0u8;
  println!("Initializing virtio block device with ioport {:?}..", port);
  port.write8(18, state);

  state = state | VIRTIO_STATUS_ACKNOWLEDGE;
  port.write8(18, state);

  state = state | VIRTIO_STATUS_DRIVER;
  port.write8(18, state);

  // Feature negotiation
  let offered_featureflags = port.read16(0);
  println!("The device offered us these feature bits: {:?}", offered_featureflags);
  // In theory, we'd do `negotiated = offered & supported`; we don't actually
  // support any flags, so we can just set 0.
  port.write16(4, 0);

  // Now comes the block-device-specific setup.
  // (The configuration of a single virtqueue isn't device-specific though; it's the same
  // for i.e. the virtio network controller)

  // Discover virtqueues; the block devices only has one
  if port.read16(4) != 0 {
    return Err(Error::VirtioHandshakeFailure)
  }
  // initialize the first (and only, for block devices) virtqueue
  let mut q = queue::Virtqueue::new(0, &mut port);
  // Tell the device we're done setting it up
  state = state | VIRTIO_STATUS_DRIVER_OK;
  port.write8(18, state);

  println!("Device state is now: {}", state);

  // if q.test(&port) {
  //   println!("Virtio-blk device successfully initialized and tested!");
  // } else {
  //   panic!("Self-test failed!")
  // }

  Ok(Blockdev { q: q, port: port })
}
