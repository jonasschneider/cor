use prelude::*;
use mem::*;

use cpuio;

use super::vring;
use super::vring::Descriptor;

use sched;
use collections::btree_map::BTreeMap;
use sync::global_mutex::GlobalMutex;

const VRING_DESC_F_NEXT: u16 = 1; /* This marks a buffer as continuing via the next field. */
const VRING_DESC_F_WRITE: u16 = 2; /* This marks a buffer as write-only (otherwise read-only). */

// TODO: multi-scatter buffers for FS
// TODO: can we keep all these things private?
#[derive(Debug)]
pub enum Buf {
  Simple(u16, Box<[u8]>),
  Rww(u16, Box<[u8]>, u16, Box<[u8]>, u16, Box<[u8]>),
}

pub type CondvarWait = sched::blocking::WaitToken;
type CondvarSignal = sched::blocking::SignalToken;

// Handles receive notifications for a virtio device.
pub struct RxHandler {
  pub isr_status_port: cpuio::IoPort,

  // The rings to receive on
  pub rings: Vec<Rx>,
}

extern {
  fn asm_eoi();
}

impl Debug for RxHandler {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<RxHandler>")
  }
}

impl sched::irq::InterruptHandler for RxHandler {
  fn critical(&mut self) {
    // The virtio IRQ status is reset by **reading** from this port
    if self.isr_status_port.read8(19) & 1 == 0  {
      println!("ISR==0, this interrupt likely wasn't for us.");
      return;
    }

    for ref mut rx in &mut self.rings {
      rx.check();
    }

    unsafe { asm_eoi(); } // TODO
  }

  fn noncritical(&self) {
    println!("NONcritical self: {:?}", self);
  }
}

pub type Handler = Box<FnMut(&mut VecDeque<(Buf, usize)>, &GlobalMutex<VecDeque<Buf>>) -> () + Send>;

pub struct Rx {
  used: vring::Used,
  used_buffers: Arc<GlobalMutex<VecDeque<(Buf, usize)>>>,
  inflight_buffers: Arc<GlobalMutex<BTreeMap<u16, Buf>>>, // TODO(perf): array-based map is probably faster
  device_activity: CondvarSignal,

  // I don't *really* want this here, but otherwise you can't access the Virtq
  // from the callback handler.
  free_buffers: Arc<GlobalMutex<VecDeque<Buf>>>,

  // Do something with the `used` vec, after some things have been added to it.
  process_used: Box<FnMut(&mut VecDeque<(Buf, usize)>, &GlobalMutex<VecDeque<Buf>>) -> () + Send>,
    // for blockdev: read wait tokens, wake up accordingly
    // for chardev-tx: just put back buffer into free_buffers
    // for chardev-rx: put into chardev.unread_buffers
}

impl Rx {
  fn check(&mut self) {
    println!("Checking some ring.");
    let mut any = false;
    let mut used = self.used_buffers.lock();
    println!("Got teh spinlock");

    while let Some((ref descid, ref written)) = self.used.take_from_ring() {
      any = true;
      println!("Took buffer {:?} with {} written", descid, written);
      let buf = self.inflight_buffers.lock().remove(descid).unwrap();
      used.push_back((buf,*written));
    }

    if any {
      println!("Took some buffers, calling process_used()..");
      self.process_used.call_mut((&mut *used,&*self.free_buffers));
      self.device_activity.signal(); // TODO: notify_all instead of notify_one
    } else {
      println!("Had no buffers to take.");
    }
  }
}

#[derive(Debug)]
pub struct Virtq {
  avail: vring::Avail,

  // this is the only field that should really be public
  pub device_activity: CondvarWait,

  pub used_buffers: Arc<GlobalMutex<VecDeque<(Buf, usize)>>>,
  inflight_buffers: Arc<GlobalMutex<BTreeMap<u16, Buf>>>,

  pub free_buffers: Arc<GlobalMutex<VecDeque<Buf>>>,
  pub free_descriptors: VecDeque<u16>,

  index: u16,
}

impl Virtq {
  pub fn send(&mut self, data: &[u8], port: &mut cpuio::IoPort) -> Option<usize> {
    // panic when either no buffer is available at all, or the available one isn't Simple
    match self.free_buffers.lock().pop_front() {
      Some(Buf::Simple(descriptor_id, mut buf)) => {
        let n = buf.clone_from_slice(data);

        // careful: need to add to inflight before adding to ring.
        // Also, make sure that we're not overriding any other in-flight entry.
        assert!(self.inflight_buffers.lock().insert(descriptor_id, Buf::Simple(descriptor_id, buf)).is_none());
        self.avail.add_to_ring(descriptor_id);

        // Notify, TODO: make this optional
        port.write16(16, self.index);

        Some(n)
      },
      _ => { panic!("no suitable buffer found for send"); }
    }
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
    self.free_buffers.lock().push_back(Buf::Simple(i, mem));
  }

  pub fn register_and_send_rww(&mut self, hdr: Box<[u8]>, data: Box<[u8]>, done: Box<[u8]>) -> u16 {
    let i1 = self.free_descriptors.pop_front().unwrap();
    let i2 = self.free_descriptors.pop_front().unwrap();
    let i3 = self.free_descriptors.pop_front().unwrap();

    self.avail.write_descriptor_at(i1 as usize, Descriptor {
      addr: physical_from_kernel(hdr.as_ptr() as usize) as u64,
      len: hdr.len() as u32,
      flags: VRING_DESC_F_NEXT,
      next: i2,
    });

    self.avail.write_descriptor_at(i2 as usize, Descriptor {
      addr: physical_from_kernel(data.as_ptr() as usize) as u64,
      len: data.len() as u32,
      flags: VRING_DESC_F_NEXT | VRING_DESC_F_WRITE,
      next: i3,
    });

    self.avail.write_descriptor_at(i3 as usize, Descriptor {
      addr: physical_from_kernel(done.as_ptr() as usize) as u64,
      len: done.len() as u32,
      flags: VRING_DESC_F_WRITE,
      next: 0,
    });

    let buf = Buf::Rww(i1, hdr, i2, data, i3, done);

    // no overwrite, and add to inflight before adding to ring
    assert!(self.inflight_buffers.lock().insert(i1, buf).is_none());

    // todo: need to think harder about wraparound and tag uniqueness
    i1
  }

  pub fn xx(&mut self, i1: u16, port: &mut cpuio::IoPort)  {
    self.avail.add_to_ring(i1);

    // Notify, TODO: make this optional
    port.write16(16, self.index);
  }

  // queue_index is the index on the virtio device to initialize
  pub fn new(queue_index: u16, port: &mut cpuio::IoPort, process: Box<FnMut(&mut VecDeque<(Buf, usize)>, &GlobalMutex<VecDeque<Buf>>,) -> () + Send>) -> (Self, Rx) {
    // Set queue_select
    port.write16(14, queue_index);

    // Determine how many descriptors the queue has, and allocate memory for the
    // descriptor table and the ring arrays.
    let length = port.read16(12);
    assert!(length > 0);

    let (address, mut availring, mut usedring) = vring::setup(length);

    let physical_32 = physical_from_kernel(address as usize) as u32; // FIXME: not really a safe cast
    port.write32(8, physical_32 >> 12);

    let (wait, signal) = sched::blocking::tokens(String::new());

    let used = Arc::new(GlobalMutex::new(VecDeque::new()));
    let free = Arc::new(GlobalMutex::new(VecDeque::new()));
    let inf = Arc::new(GlobalMutex::new(BTreeMap::new()));

    let rx = Rx {
      used: usedring,
      used_buffers: used.clone(),
      inflight_buffers: inf.clone(),
      device_activity: signal,

      process_used: process,
      free_buffers: free.clone(),
    };

    // set initial free descriptor list
    let mut descs = VecDeque::with_capacity(length as usize);
    for i in 0..length {
      descs.push_back(i);
    }

    (Virtq {
      avail: availring,
      device_activity: wait,
      used_buffers: used,

      free_buffers: free,
      free_descriptors: descs,
      inflight_buffers: inf,
      index: queue_index,
    }, rx)
  }
}
