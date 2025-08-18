//! Game
//!
//! This crate provides types related to chess game representation and search.

pub use game::{Game, GameEnding, GameSearch};
pub use transposition_table::{
    CollisionResult, Entry, EntryCollision, OccupiedEntry, TranspositionTable, VacantEntry,
};

mod game;
mod transposition_table;

#[cfg(test)]
mod perft;

/*
    TO BENCHMARK:
    - bitboard representation (classical / denser)
    - inlining (???)
*/
