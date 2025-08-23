//! Game
//!
//! This crate provides types related to chess game representation and search.

pub use hashmap::{ClashEntry, EmptyEntry, Entry, ExactEntry, KeyLookup, WeakHashMap};
pub use search::{ParallelSearch, Score, SearchResult};
pub use transposition::{Transposition, TranspositionTable};
pub use waiter::Waiter;

mod hashmap;
mod search;
mod transposition;
mod waiter;

#[cfg(test)]
mod perft;

/*
    TO BENCHMARK:
    - bitboard representation (classical / denser)
    - inlining (???)
*/
