//! Game
//!
//! This module provedes types related to chess game representation and search.

/*
    TO BENCHMARK:
    - bitboard representation (classical / denser)
    - inlining (???)
*/
pub mod game;
pub mod transposition_table;

#[cfg(test)]
mod perft;
