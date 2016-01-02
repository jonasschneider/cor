// inspired by:
// https://github.com/rust-lang/rust/blob/377b0900aede976b2d37a499bbd7b62c2e39b358/src/libstd/sync/mpsc/blocking.rs

use prelude::*;
use sched;

#[derive(Debug)]
struct Inner {
  woken: AtomicBool,
  name: String,
}

unsafe impl Send for Inner {}
unsafe impl Sync for Inner {} // we don't really need it

use core::sync::atomic::{AtomicBool,ATOMIC_BOOL_INIT,AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
static NEXT_TOKEN: AtomicUsize = ATOMIC_USIZE_INIT;

#[derive(Clone)]
#[derive(Debug)]
pub struct SignalToken {
  inner: Arc<Inner>,
}

// Fixme: this Clone is dubious
#[derive(Clone)]
#[derive(Debug)]
pub struct WaitToken {
  inner: Arc<Inner>,
}

// TODO: actually park/unpark (to avoid unnecessary wakeups in the wait loop)
pub fn tokens(desc: String) -> (WaitToken, SignalToken) {
  let inner = Arc::new(Inner {
      woken: AtomicBool::new(false),
      name: desc,
  });
  let wait_token = WaitToken {
      inner: inner.clone(),
  };
  let signal_token = SignalToken {
      inner: inner
  };
  (wait_token, signal_token)
}

impl SignalToken {
  pub fn signal(&self) -> bool {
    let wake = !self.inner.woken.compare_and_swap(false, true, Ordering::SeqCst);
    // if wake {
    //     self.inner.thread.unpark();
    // }
    wake
  }
}

impl WaitToken {
  pub fn wait(self) {
    while !self.inner.woken.load(Ordering::SeqCst) {
      //sched::park_until_irq(0x2b);
      println!("Wait token {:?} isn't woken yet, sleeping..",&self);
      sched::kyield();
    }
    println!("Woke wait token {:?}", &self);
  }

  pub fn multiwait(&mut self) {
    while !self.inner.woken.compare_and_swap(true, false, Ordering::SeqCst) {
      //sched::park_until_irq(0x2b);
      println!("Wait token {:?} isn't woken yet, sleeping..",&self);
      sched::kyield();
    }
    println!("Woke wait token {:?}, and put it back to sleep", &self);
  }
}
