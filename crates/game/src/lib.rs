//! Game
//!
//! This crate provides types related to chess game representation and search.

pub use game::{Game, GameEnding, GameSearch};
pub use hashmap::{
    WeakHashMap, KeyLookup, Entry, EmptyEntry, ClashEntry, ExactEntry,
};

mod game;
mod hashmap;

#[cfg(test)]
mod perft;

/*
    TO BENCHMARK:
    - bitboard representation (classical / denser)
    - inlining (???)
*/
