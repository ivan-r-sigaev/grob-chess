//! Board
//!
//! This crate provides types related to board representation.

pub use bitboard::BitBoard;
pub use board::Board;
pub use pieces::{Color, Piece, Promotion};
pub use square::{File, NegDiag, PosDiag, Rank, Square};

mod bitboard;
mod board;
mod move_calculation;
mod pieces;
mod square;
