//! `position` provides chess position representation and move generation.

/*
    TO BENCHMARK:
    - bitboard representation (classical / denser)
    - inlining (???)
*/

pub mod prelude;

pub mod board;
pub mod position;

mod bitboard;
mod castling_rights;
mod move_calculation;
mod move_generation;
mod pieces;
mod position_hash;
mod square;
