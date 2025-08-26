use std::sync::{
    atomic::{AtomicU8, Ordering},
    Arc, Barrier,
};

use crossbeam::utils::CachePadded;

const SIGNAL_GO: u8 = 0;
const SIGNAL_STOP: u8 = 1;
const SIGNAL_QUIT: u8 = 2;

#[derive(Debug)]
pub struct WorkerSignalerMaster {
    signal: Arc<CachePadded<AtomicU8>>,
    wakeup: Arc<Barrier>,
    sleep: Arc<Barrier>,
    worker_count: usize,
    clone_count: usize,
}

impl WorkerSignalerMaster {
    pub fn new(worker_count: usize) -> Self {
        Self {
            signal: Arc::new(CachePadded::new(AtomicU8::new(SIGNAL_STOP))),
            wakeup: Arc::new(Barrier::new(worker_count + 1)),
            sleep: Arc::new(Barrier::new(worker_count + 1)),
            worker_count,
            clone_count: 0,
        }
    }
    pub fn create_signaler(&mut self) -> WorkerSignaler {
        self.clone_count += 1;
        assert!(
            self.clone_count <= self.worker_count,
            "Trying to create too many workers!"
        );
        WorkerSignaler {
            signal: self.signal.clone(),
            wakeup: self.wakeup.clone(),
            sleep: self.sleep.clone(),
        }
    }
    pub fn is_running(&self) -> bool {
        self.signal.load(Ordering::Relaxed) == SIGNAL_GO
    }
    pub fn worker_count(&self) -> usize {
        self.worker_count
    }
    pub fn go(&self) {
        self.check_workers();
        if self.is_running() {
            self.stop();
        }

        self.signal.store(SIGNAL_GO, Ordering::Relaxed);
        self.wakeup.wait();
    }
    pub fn stop(&self) {
        self.check_workers();
        if !self.is_running() {
            return;
        }

        self.signal.store(SIGNAL_STOP, Ordering::Relaxed);
        self.sleep.wait();
    }
    pub fn quit(self) {
        self.check_workers();
        if self.is_running() {
            self.stop();
        }

        self.signal.store(SIGNAL_QUIT, Ordering::Relaxed);
        self.wakeup.wait();
    }
    fn check_workers(&self) {
        assert!(self.clone_count == self.worker_count, "Not enough workers!");
    }
}

#[derive(Debug)]
pub struct WorkerSignaler {
    signal: Arc<CachePadded<AtomicU8>>,
    wakeup: Arc<Barrier>,
    sleep: Arc<Barrier>,
}

impl WorkerSignaler {
    pub fn should_stop(&self) -> bool {
        self.signal.load(Ordering::Relaxed) == SIGNAL_STOP
    }
    pub fn should_quit(&self) -> bool {
        self.signal.load(Ordering::Relaxed) == SIGNAL_QUIT
    }
    pub fn wakeup(&self) {
        self.wakeup.wait();
    }
    pub fn sleep(&self) {
        self.sleep.wait();
    }
}
