use std::time::Instant;

use crossbeam::channel::{Receiver, Select};

/// This is a convenience wrapper around `crossbeam_channel::select::Select`.
///
/// Like `crossbeam_channel::select::Select` the purpose of the [`Waiter`]
/// is to block on multiple `Receiver`s until one of them becomes ready.
/// [`Waiter`] can also keep the track of the deadlines for each of the added
/// receivers if those receivers have to stop blocking upon reaching them.
#[derive(Debug, Clone)]
pub struct Waiter<'a> {
    sel: Select<'a>,
    deadline: Option<(Instant, usize)>,
}

impl<'a> Waiter<'a> {
    /// Constructs a new empty [`Waiter`].
    pub fn new() -> Self {
        Self {
            sel: Select::new(),
            deadline: None,
        }
    }
    /// Adds the receiver to be tracked by the waiter and returs its index.
    ///
    /// If deadline is specified the waiter will act as if the added
    /// receiver is ready upon reaching this deadline.
    pub fn add<T>(&mut self, r: &'a Receiver<T>, deadline: Option<Instant>) -> usize {
        let index = self.sel.recv(r);
        if let Some(deadline) = deadline {
            if self.deadline.is_none_or(|(d, _)| deadline < d) {
                self.deadline = Some((deadline, index));
            }
        }
        index
    }
    /// Blocks until one of the added receivers is ready or
    /// have reached its deadline and returns the index of this receiver.
    ///
    /// # Panics
    /// Panics if no receivers were added.
    pub fn wait(mut self) -> usize {
        if let Some((deadline, index)) = self.deadline {
            self.sel.ready_deadline(deadline).unwrap_or(index)
        } else {
            self.sel.ready()
        }
    }
}

impl<'a> Default for Waiter<'a> {
    fn default() -> Self {
        Self::new()
    }
}
