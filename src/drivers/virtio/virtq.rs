use prelude::*;
use mem::*;

use core::borrow::{BorrowMut,Borrow};
use cpuio;

use super::vring;
use super::vring::Descriptor;

use sched;
use collections::btree_map::BTreeMap;
use sync::global_mutex::GlobalMutex;

const VRING_DESC_F_NEXT: u16 = 1; /* This marks a buffer as continuing via the next field. */
const VRING_DESC_F_WRITE: u16 = 2; /* This marks a buffer as write-only (otherwise read-only). */

// might also be a buffer with several segments.. bleh
#[derive(Debug)]
struct Buf(u16, Box<[u8]>);
// ID is accessible and unique *for the lifetime of the LogicalBuf*

type CondvarWait = sched::blocking::WaitToken;
type CondvarSignal = sched::blocking::SignalToken;

// Handles receive notifications for a virtio device.
#[derive(Debug)]
pub struct RxHandler {
  pub isr_status_port: cpuio::IoPort,

  // The rings to receive on
  pub rings: Vec<Rx>,
}

extern {
  fn asm_eoi();
}

impl sched::irq::InterruptHandler for RxHandler {
  fn critical(&mut self) {
    unsafe { asm_eoi(); } // TODO
    // The virtio IRQ status is reset by **reading** from this port
    if self.isr_status_port.read8(19) & 1 == 0  {
      println!("ISR==0, this interrupt likely wasn't for us.");
      return;
    }

    for ref mut rx in &mut self.rings {
      rx.check();
    }


  }

  fn noncritical(&self) {
    println!("NONcritical self: {:?}", self);
  }
}


#[derive(Debug)]
pub struct Rx {
  used: vring::Used,
  inflight_buffers: Arc<GlobalMutex<BTreeMap<u16, Buf>>>,
  device_activity: CondvarSignal,

  //process_used: Box<Fn(Buf, usize) -> Option<Buf>>,
    // if you take the Buf for yourself, make sure that you deregister it before dropping.
    // for blockdev: read wait tokens, wake up accordingly
    // for chardev-tx: just put back buffer into free_buffers
    // for chardev-rx: put into chardev.unread_buffers
}

impl Rx {
  fn check(&mut self) {

  }
  fn process_used(buf: Buf, written: usize) -> Option<Buf> {
    Some(buf)
  }
}

#[derive(Debug)]
pub struct Virtq {
  avail: vring::Avail,

  device_activity: CondvarWait,
  used_buffers: Arc<GlobalMutex<VecDeque<Buf>>>,
  inflight_buffers: Arc<GlobalMutex<BTreeMap<u16, Buf>>>,

  free_buffers: VecDeque<Buf>,
  free_descriptors: VecDeque<u16>,
}

impl Virtq {
  pub fn send(&mut self, data: &[u8], port: &mut cpuio::IoPort) -> Option<usize> {
    let Buf(descriptor_id, mut buf) = self.free_buffers.pop_front().unwrap(); // panic on no available buf
    let n = buf.clone_from_slice(data);
    self.avail.add_to_ring(descriptor_id);
    println!("enqueued it: {:?}", &self.avail.mem[128*16..]);

    println!("Waiting for take() on used.. XX");


    loop {
      port.write16(16, 1);
    }
    // while let None = self.used.take_from_ring() {

    // }
    // println!("done");
  }

  pub fn register(&mut self, mem: Box<[u8]>, device_writable: bool) {
    let i = self.free_descriptors.pop_front().unwrap();
    let flags = if device_writable { VRING_DESC_F_WRITE } else { 0 };
    self.avail.write_descriptor_at(i as usize, Descriptor {
      addr: physical_from_kernel((mem[..]).as_ptr() as usize) as u64,
      len: mem.len() as u32,
      flags: flags,
      next: 0,
    });
    self.free_buffers.push_back(Buf(i, mem));
  }

  // queue_index is the index on the virtio device to initialize
  pub fn new(queue_index: u16, port: &mut cpuio::IoPort) -> (Self, Rx) {
    // Set queue_select
    port.write16(14, queue_index);

    // Determine how many descriptors the queue has, and allocate memory for the
    // descriptor table and the ring arrays.
    let length = port.read16(12);

    let (address, mut avail, mut used) = vring::setup(length);

    let physical_32 = physical_from_kernel(address as usize) as u32; // FIXME: not really a safe cast
    port.write32(8, physical_32 >> 12);

    let (wait, signal) = sched::blocking::tokens(String::new());

    let free = Arc::new(GlobalMutex::new(VecDeque::new()));
    let inf = Arc::new(GlobalMutex::new(BTreeMap::new()));

    let rx = Rx {
      used: used,
      inflight_buffers: inf.clone(),
      device_activity: signal,
      // process_used: box move |buf, written| {
      //   println!("process_used: written={}, {:?}", written, buf);
      //   Some(buf)
      // },
    };

    let mut descs = VecDeque::with_capacity(length as usize);
    for i in 0..length {
      descs.push_back(i);
    }

    (Virtq {
      avail: avail,
      device_activity: wait,
      used_buffers: free,

      free_buffers: VecDeque::new(),
      free_descriptors: descs,
      inflight_buffers: inf,
    }, rx)
  }
}
