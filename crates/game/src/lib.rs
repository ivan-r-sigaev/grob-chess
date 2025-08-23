//! Position
//!
//! This crate provides types related to game representation and move generation.

pub use bitboard::BitBoard;
pub use board::Board;
pub use castling_rights::CastlingRights;
pub use game::{Game, GameEnding, GameExplorer, MoveOrdering};
pub use pieces::{Color, Piece, Promotion};
pub use position::{ChessMove, ChessMoveHint, LanMove, PackedChessMove};
pub use raw_position::ParseFenError;
pub use square::{File, NegDiag, PosDiag, Rank, Square};

mod bitboard;
mod board;
mod castling_rights;
mod game;
mod move_calculation;
mod move_list;
mod pieces;
mod position;
mod raw_position;
mod square;
mod zobrist;

#[cfg(test)]
mod perft;

/*
    TO BENCHMARK:
    - bitboard representation (classical / denser)
    - inlining (???)
*/
