//! Game
//!
//! This crate provides types related to chess game representation and search.

pub use game::{Game, GameEnding, GameSearch};
pub use hashmap::{ClashEntry, EmptyEntry, Entry, ExactEntry, KeyLookup, WeakHashMap};
pub use transposition::{Transposition, TranspositionTable};
pub use search::{ParallelSearch, Score, SearchResult};
pub use waiter::Waiter;

mod game;
mod search;
mod waiter;
mod hashmap;
mod transposition;

#[cfg(test)]
mod perft;

/*
    TO BENCHMARK:
    - bitboard representation (classical / denser)
    - inlining (???)
*/
