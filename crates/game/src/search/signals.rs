use std::sync::{
    atomic::{AtomicU8, Ordering},
    Arc, Barrier,
};

use crossbeam::utils::CachePadded;

const SIGNAL_GO: u8 = 0;
const SIGNAL_STOP: u8 = 1;
const SIGNAL_QUIT: u8 = 2;

/// Signaler that can crate and control [`WorkerSignaler`]s.
#[derive(Debug)]
pub struct WorkerSignalerMaster {
    signal: Arc<CachePadded<AtomicU8>>,
    wakeup: Arc<Barrier>,
    sleep: Arc<Barrier>,
    worker_count: usize,
    clone_count: usize,
}

impl WorkerSignalerMaster {
    /// Construct a new [`WorkerSignalerMaster`] able to control
    /// the specified number of workers.
    pub fn new(worker_count: usize) -> Self {
        Self {
            signal: Arc::new(CachePadded::new(AtomicU8::new(SIGNAL_STOP))),
            wakeup: Arc::new(Barrier::new(worker_count + 1)),
            sleep: Arc::new(Barrier::new(worker_count + 1)),
            worker_count,
            clone_count: 0,
        }
    }
    /// Construct a child [`WorkerSignaler`].
    ///
    /// # Panics
    /// Panics when trying to create more than [`Self::worker_count`]
    /// signalers.
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
    /// Returns `true` if since the last call to [`Self::go`]
    /// there were no calls to [`Self::stop`].
    ///
    /// This will return `false` if the [`Self::go`] was
    /// never called yet.
    pub fn is_running(&self) -> bool {
        self.signal.load(Ordering::Relaxed) == SIGNAL_GO
    }
    /// Returns the exact amount of workers supported
    /// (and also minimally required) by the signaler.
    pub fn worker_count(&self) -> usize {
        self.worker_count
    }
    /// Tells the workers to start doing their jobs.
    ///
    /// This does nothing if workers are already awake.
    ///
    /// # Panics
    /// Panics the number of created [`WorkerSignaler`]s
    /// is not exactly equal to [`Self::worker_count`].
    pub fn go(&self) {
        self.check_workers();
        if self.is_running() {
            return;
        }

        self.signal.store(SIGNAL_GO, Ordering::Relaxed);
        self.wakeup.wait();
    }
    /// Tells the workers to stop doing their jobs and go to sleep.
    ///
    /// This does nothing if workers are already sleeping (except for
    /// setting [`Self::is_running`] to `false`).
    ///
    /// # Panics
    /// Panics the number of created [`WorkerSignaler`]s
    /// is not exactly equal to [`Self::worker_count`].
    pub fn stop(&self) {
        self.check_workers();
        if !self.is_running() {
            return;
        }

        self.signal.store(SIGNAL_STOP, Ordering::Relaxed);
        self.sleep.wait();
    }
    /// [`Drop`]s the master and tells all of its children to quit.
    ///
    /// [`WorkerSignalerMaster`] will **NOT** automatically tell
    /// the workers to quit on [`Drop`].
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

/// A child of the [`WorkerSignalerMaster`].
///
/// This struct is used to coordinate workers during search.
#[derive(Debug)]
pub struct WorkerSignaler {
    signal: Arc<CachePadded<AtomicU8>>,
    wakeup: Arc<Barrier>,
    sleep: Arc<Barrier>,
}

impl WorkerSignaler {
    /// Returns whether the master is commanding to stop.
    pub fn should_stop(&self) -> bool {
        self.signal.load(Ordering::Relaxed) == SIGNAL_STOP
    }
    /// Returns whether the master is commanding to quit.
    pub fn should_quit(&self) -> bool {
        self.signal.load(Ordering::Relaxed) == SIGNAL_QUIT
    }
    /// Call this in the beginning of the worker loop.
    pub fn wakeup(&self) {
        self.wakeup.wait();
    }
    /// Call this at the end of the worker loop.
    pub fn sleep(&self) {
        self.sleep.wait();
    }
}
