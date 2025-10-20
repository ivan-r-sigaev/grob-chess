//! Grob Core
//!
//! This crate provides the basic chess types.

pub mod bitboard;
pub mod board;
pub mod castling_rights;
pub mod game;
pub mod pieces;
pub mod square;

mod position;

/*
    TO BENCHMARK:
    - bitboard representation (classical / denser)
    - inlining (???)
*/
