mod types;
mod queue;

use cpuio;

use kalloc::__rust_allocate as allocate;
use alloc::boxed::Box;
use core;
use core::slice;
use core::fmt;
use kbuf;
use collections;
use collections::vec::Vec;
use super::sched;


#[derive(Debug)]
pub struct Device {
  io_base: cpuio::Port,
}

#[derive(Debug)]
pub enum Error {
  VirtioHandshakeFailure,
  NoDiskMarker,
}

const VIRTIO_STATUS_ACKNOWLEDGE: u8 = 1;
const VIRTIO_STATUS_DRIVER: u8 = 2;
const VIRTIO_STATUS_DRIVER_OK: u8 = 4;
const VIRTIO_STATUS_FAILED: u8 = 128;


pub unsafe fn init(port: cpuio::IoPort) -> Result<Device, Error> {
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
  let mut q = queue::Virtqueue::new(0, &port);
  // Tell the device we're done setting it up
  state = state | VIRTIO_STATUS_DRIVER_OK;
  port.write8(18, state);

  println!("Device state is now: {}", state);

  if q.test(&port) {
    println!("Virtio-blk device successfully initialized and tested!");
  } else {
    panic!("Self-test failed!")
  }

  Ok(Device { io_base: 0 })
}
