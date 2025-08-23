//! Position
//!
//! This crate provides types related to position representation and move generation.

pub use castling_rights::CastlingRights;
pub use move_generation::{ChessMove, ChessMoveHint, ChessUnmove, LanMove, PackedChessMove};
pub use move_list::MoveList;
pub use position::{ParseFenError, Position};
pub use position_hash::PositionHash;

mod castling_rights;
mod move_generation;
mod move_list;
mod position;
mod position_hash;
mod zobrist;
