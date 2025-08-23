//! Position
//!
//! This crate provides types related to position representation and move generation.

pub use castling_rights::CastlingRights;
pub use game::{Game, GameEnding, GameExplorer};
pub use move_list::MoveList;
pub use position::{ChessMove, ChessMoveHint, LanMove, PackedChessMove};
pub use raw_position::ParseFenError;

mod castling_rights;
mod game;
mod move_list;
mod position;
mod raw_position;
mod zobrist;

#[cfg(test)]
mod perft;

/*
    TO BENCHMARK:
    - bitboard representation (classical / denser)
    - inlining (???)
*/
