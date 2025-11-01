use std::sync::{
    Arc, Barrier,
    atomic::{AtomicU8, Ordering},
};

use crossbeam::utils::CachePadded;

const SIGNAL_QUIT: u8 = 0;
const SIGNAL_STOP: u8 = 1;
const SIGNAL_GO: u8 = 2;

/// Tells the worker threads when to start and when to stop doing work.
#[derive(Debug)]
pub struct SignalSender {
    channel: Arc<SignalChannel>,
}

impl SignalSender {
    /// Returns a sender - receivers connection for the specified
    /// number of receivers.
    pub fn new(receiver_count: usize) -> (SignalSender, Vec<SignalReceiver>) {
        let channel = Arc::new(SignalChannel {
            signal: CachePadded::new(AtomicU8::new(SIGNAL_STOP)),
            wakeup: Barrier::new(receiver_count + 1),
            sleep: Barrier::new(receiver_count + 1),
        });
        let receivers = (0..receiver_count)
            .map(|_| SignalReceiver {
                channel: channel.clone(),
            })
            .collect();
        let sender = SignalSender { channel };
        (sender, receivers)
    }
    /// Returns `true` if worker threads are currently awake.
    pub fn is_running(&self) -> bool {
        self.channel.load_signal() == SIGNAL_GO
    }
    /// Returns `true` if worker threads are currently sleeping.
    pub fn is_stopped(&self) -> bool {
        self.channel.load_signal() == SIGNAL_STOP
    }
    /// Wakes up the worker threads unless they are already awake.
    pub fn go(&self) {
        if self.is_running() {
            return;
        }

        self.channel.store_signal(SIGNAL_GO);
        self.channel.wakeup();
    }
    /// Tells the worker threads to go to sleep unless they are already sleeping.
    pub fn stop(&self) {
        if self.is_stopped() {
            return;
        }

        self.channel.store_signal(SIGNAL_STOP);
        self.channel.sleep();
    }
    /// Terminates the worker threads and consumes the sender.
    ///
    /// If the workers are awake, [`SignalSender::stop`] will be called first.
    ///
    /// This function must be called manually and will **NOT**
    /// be automatically called on [`Drop`].
    pub fn quit(self) {
        self.stop();
        self.channel.store_signal(SIGNAL_QUIT);
        self.channel.wakeup();
    }
}

/// Recieves the messages when the worker must start or stop doing work.
#[derive(Debug)]
pub struct SignalReceiver {
    channel: Arc<SignalChannel>,
}

impl SignalReceiver {
    /// Returns `true` if search should be aborted.
    pub fn should_stop(&self) -> bool {
        self.channel.load_signal() == SIGNAL_STOP
    }
    /// Must be called at the start of worker loop.
    ///
    /// Returns `false` if the worker thread must quit.
    #[must_use]
    pub fn go(&self) -> bool {
        self.channel.wakeup();
        self.channel.load_signal() == SIGNAL_QUIT
    }
    /// Must be called at the end of worker loop.
    ///
    /// # Panics
    /// Panics if [`Self::go`] previously returned `false`.
    pub fn stop(&self) {
        assert!(
            self.channel.load_signal() != SIGNAL_QUIT,
            "Worker ignored termination request!"
        );
        self.channel.sleep();
    }
}

/// Underlying type for the signal channels.
#[derive(Debug)]
struct SignalChannel {
    signal: CachePadded<AtomicU8>,
    wakeup: Barrier,
    sleep: Barrier,
}

impl SignalChannel {
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
    /// Waits until all workers go to sleep.
    fn sleep(&self) {
        _ = self.sleep.wait();
    }
    /// Waits until all workers wake up from sleep.
    fn wakeup(&self) {
        _ = self.wakeup.wait();
    }
}
