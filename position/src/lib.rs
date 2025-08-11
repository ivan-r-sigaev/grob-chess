/*
    TO BENCHMARK:
    - bitboard representation (classical / denser)
    - inlining (???)
*/
pub mod prelude;

pub mod board;
pub mod pieces;
pub mod position;
pub mod square;

mod bitboard;
mod castling_rights;
mod move_calculation;
mod move_generation;
mod position_hash;
