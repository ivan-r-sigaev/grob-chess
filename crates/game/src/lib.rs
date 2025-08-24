//! Position
//!
//! This crate provides types related to game representation and move generation.

pub use bitboard::BitBoard;
pub use board::Board;
pub use castling_rights::CastlingRights;
pub use game::{Game, GameEnding, GameExplorer, MoveOrdering};
pub use pieces::{Color, Piece, Promotion};
pub use position::{ChessMove, ChessMoveHint, LanMove, PackedChessMove};
pub use position_base::ParseFenError;
pub use search::{ParallelSearch, Score, SearchResult};
pub use square::{File, NegDiag, PosDiag, Rank, Square};
pub use transposition::{Transposition, TranspositionTable};

mod bitboard;
mod board;
mod cache;
mod castling_rights;
mod game;
mod move_calculation;
mod move_list;
mod pieces;
mod position;
mod position_base;
mod search;
mod square;
mod transposition;
mod zobrist;

#[cfg(test)]
mod perft;

/*
    TO BENCHMARK:
    - bitboard representation (classical / denser)
    - inlining (???)
*/
