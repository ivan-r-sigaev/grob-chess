//! Position
//!
//! This crate provides types related to position representation and move generation.

pub use castling_rights::CastlingRights;
pub use move_generation::{
    ChessMove, ChessMoveHint, ChessUnmove, GameEnding, GameSearch, LanMove, PackedChessMove,
};
pub use move_list::MoveList;
pub(crate) use position::PlyHistory;
pub use position::{ParseFenError, Position};

mod castling_rights;
mod move_generation;
mod move_list;
mod position;
mod zobrist;
