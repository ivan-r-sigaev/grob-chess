//! Game
//!
//! This crate provides types related to chess game representation and search.

pub use hashmap::{ClashEntry, EmptyEntry, Entry, ExactEntry, KeyLookup, WeakHashMap};
pub use search::{ParallelSearch, Score, SearchResult};
pub use transposition::{Transposition, TranspositionTable};

mod hashmap;
mod search;
mod transposition;
