use prelude::*;
use cpuio;
use super::virtq;
use sync::global_mutex::GlobalMutex;
use sched;

use core::iter::Map;

const VIRTIO_STATUS_ACKNOWLEDGE: u8 = 1;
const VIRTIO_STATUS_DRIVER: u8 = 2;
const VIRTIO_STATUS_DRIVER_OK: u8 = 4;
const VIRTIO_STATUS_FAILED: u8 = 128;

pub fn init(mut port: cpuio::IoPort, irqnum: u8, rxhandlers: Vec<(u16, virtq::Handler)>) -> (Vec<virtq::Virtq>, cpuio::IoPort) {
  println!("Initializing virtio device on ioport {:?}..", port);

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

  let mut rxs = vec![];
  let mut qs = vec![];

  for (queue_index, handler) in rxhandlers {
    let (q, rx) = virtq::Virtq::new(queue_index, &mut txport, handler);
    qs.push(q);
    rxs.push(rx);
  }

  let handler = virtq::RxHandler {
    rings: rxs,
    isr_status_port: rxport,
  };

  sched::irq::add_handler(irqnum, box handler);

  // Tell the device we're done setting it up
  state = state | VIRTIO_STATUS_DRIVER_OK;
  configport.write8(18, state);

  (qs, txport)
}
