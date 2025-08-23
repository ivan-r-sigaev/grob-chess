//! Multithreading
//!
//! This crate provides types related to multithreading.

pub use hashmap::{ClashEntry, EmptyEntry, Entry, ExactEntry, KeyLookup, WeakHashMap};
pub use search::{ParallelSearch, Score, SearchResult};
pub use transposition::{Transposition, TranspositionTable};

mod hashmap;
mod search;
mod transposition;
