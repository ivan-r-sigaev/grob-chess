use std::sync::{Arc, Barrier, atomic::{AtomicU8, Ordering}};

use crossbeam::utils::CachePadded;

const SIGNAL_QUIT: u8 = 0;
const SIGNAL_STOP: u8 = 1;
const SIGNAL_GO: u8 = 2;

/// Synchronizes the worker threads during search.
#[derive(Debug)]
pub struct ThreadSyncMaster {
    inner: Arc<ThreadSyncInner>,
}

impl ThreadSyncMaster {
    /// Returns the synchronization primitives to controll a specified
    /// number of worker threads.
    pub fn new(worker_count: usize) -> (ThreadSyncMaster, Vec<ThreadSync>) {
        let channel = Arc::new(ThreadSyncInner {
            signal: CachePadded::new(AtomicU8::new(SIGNAL_STOP)),
            wakeup: Barrier::new(worker_count + 1),
            sleep: Barrier::new(worker_count + 1),
        });
        let receivers = (0..worker_count)
            .map(|_| ThreadSync {
                inner: channel.clone(),
            })
            .collect();
        let sender = ThreadSyncMaster { inner: channel };
        (sender, receivers)
    }
    /// Returns `true` if the worker threads are currently awake.
    pub fn is_running(&self) -> bool {
        self.inner.load_signal() == SIGNAL_GO
    }
    /// Returns `true` if the worker threads are currently sleeping.
    pub fn is_stopped(&self) -> bool {
        self.inner.load_signal() == SIGNAL_STOP
    }
    /// Wakes up the worker threads (unless they are already awake).
    pub fn go(&self) {
        if self.is_running() {
            return;
        }

        self.inner.store_signal(SIGNAL_GO);
        self.inner.wakeup.wait();
    }
    /// Forces the worker threads to abandon the search and to go to sleep 
    /// (unless they are already sleeping).
    pub fn stop(&self) {
        if self.is_stopped() {
            return;
        }

        self.inner.store_signal(SIGNAL_STOP);
        self.inner.sleep.wait();
    }
    /// Terminates the worker threads and consumes the sender.
    ///
    /// This function must be called manually and will **NOT**
    /// be automatically called on [`Drop`].
    /// 
    /// If the workers are awake, this will have the effect of 
    /// calling the [`WorkerSyncMaster::stop`] first.
    pub fn quit(self) {
        self.stop();
        self.inner.store_signal(SIGNAL_QUIT);
        self.inner.wakeup.wait();
    }
}

/// Synchronizes the worker threads during search.
#[derive(Debug)]
pub struct ThreadSync {
    inner: Arc<ThreadSyncInner>,
}

/// Instructions for thread after wakeup.
#[derive(Debug, Clone, Copy, Hash)]
pub struct ThreadWakeupResult {
    /// If this is set to `true` the thread must terminate.
    pub should_quit: bool,
    /// Only one arbitrary thread will have this set to `true` upon waking up. 
    pub is_leader: bool,
}

impl ThreadSync {
    /// Returns `true` if search should be aborted.
    pub fn should_stop(&self) -> bool {
        self.inner.load_signal() == SIGNAL_STOP
    }
    /// Must be called at the start of worker loop.
    #[must_use]
    pub fn go(&self) -> ThreadWakeupResult {
        let is_leader = self.inner.wakeup.wait().is_leader();
        let should_quit = self.inner.load_signal() == SIGNAL_QUIT;
        ThreadWakeupResult { should_quit, is_leader }
    }
    /// Must be called at the end of worker loop.
    ///
    /// # Panics
    /// Panics if [`Self::go`] previously returned `false`.
    pub fn stop(&self) {
        assert!(
            self.inner.load_signal() != SIGNAL_QUIT,
            "Worker ignored termination request!"
        );
        _ = self.inner.sleep.wait();
    }
}

/// Synchronizes the workers with the master thread.
#[derive(Debug)]
struct ThreadSyncInner {
    signal: CachePadded<AtomicU8>,
    wakeup: Barrier,
    sleep: Barrier,
}

impl ThreadSyncInner {
    /// Gets the current signal value.
    fn load_signal(&self) -> u8 {
        // Channels are syncronized with barriers, so it's OK to use [`Ordering::Relaxed.`]
        self.signal.load(Ordering::Relaxed)
    }
    /// Sets the current signal value.
    fn store_signal(&self, signal: u8) {
        // Channels are syncronized with barriers, so it's OK to use [`Ordering::Relaxed.`]
        self.signal.store(signal, Ordering::Relaxed)
    }
}
