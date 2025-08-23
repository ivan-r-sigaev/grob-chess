//! Position
//!
//! This crate provides types related to position representation and move generation.

pub use castling_rights::CastlingRights;
pub use explorer::{GameEnding, GameExplorer};
pub use game::{Game, ParseFenError};
pub use move_generation::{ChessMove, ChessMoveHint, ChessUnmove, LanMove, PackedChessMove};
pub use move_list::MoveList;

mod castling_rights;
mod explorer;
mod game;
mod move_generation;
mod move_list;
mod zobrist;

#[cfg(test)]
mod perft;

/*
    TO BENCHMARK:
    - bitboard representation (classical / denser)
    - inlining (???)
*/
