use prelude::*;

use super::virtq;
use super::pci;

use cpuio;
use sched::blocking::{WaitToken,SignalToken};
use block::{Client,Error};

use collections::btree_map::BTreeMap;
use sync::global_mutex::GlobalMutex;
use byteorder::{ByteOrder,NativeEndian};

#[derive(Debug)]
pub struct Blockdev {
  port: cpuio::IoPort,
  q: virtq::Virtq,

  completed_requests: Arc<GlobalMutex<BTreeMap<u16,virtq::Buf>>>,
}

impl Client for Blockdev {
  type Tag = u16;

  fn read_dispatch(&mut self, sector: u64, mut buf: Box<[u8]>) -> Result<Self::Tag, Error> {
    assert_eq!(512, buf.len());

    println!("virtio blockdev: Reading sector {}", sector);

    let mut hdr = box [0u8; 16];
    NativeEndian::write_u32(&mut hdr[0..], 0); // kind: 0=read
    NativeEndian::write_u32(&mut hdr[4..], 1); // ioprio
    NativeEndian::write_u64(&mut hdr[8..], sector as u64);
    let mut done = box [17u8; 1]; // != 0 for checking that it was set by the host

    let tag = self.q.register_rww(hdr, buf, done);

    self.q.send_rww(tag, &mut self.port);

    Ok(tag as Self::Tag)
  }

  fn read_await(&mut self, tag: Self::Tag) -> Result<Box<[u8]>, Error> {
    // FIXME: make sure that we're not holding any borrows or locks before we go to sleep?
    // TODO loop / condition check macro
    self.q.device_activity.clone().multiwait();

    match self.completed_requests.lock().remove(&tag) {
      // drop hdr
      Some(virtq::Buf::Rww(id1, _, id2, data, id3, done)) => {
        {
          self.q.free_descriptors.push_back(id1);
          self.q.free_descriptors.push_back(id2);
          self.q.free_descriptors.push_back(id3);
        }

        println!("Virtio call completed, retval={}", done[0]);
        if done[0] != 0 { // retval of 0 indicates success
          println!("Virtio retval is {} != 0", done[0]);
          return Err(Error::InternalError)
        }
        return Ok(data)
      }
      x => { panic!("wut! unexpected buffer type {:?}",x) }
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

impl Blockdev {
  pub fn new(mut port: cpuio::IoPort) -> Result<Self, InitError> {
    let completed = Arc::new(GlobalMutex::new(BTreeMap::new()));
    let completed_irqside = completed.clone();

    let request_completion_handler = (box move |used, free| {
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
    }) as virtq::Handler;

    let handlers = vec![(0, request_completion_handler)];
    let (mut qs, mut txport) = pci::init(port, 0x2a, handlers);

    Ok(Blockdev {
      port: txport,
      q: qs.remove(0),
      completed_requests: completed,
    })
  }
}
