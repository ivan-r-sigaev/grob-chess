//! Position
//!
//! This crate provides types related to game representation and move generation.

pub use game::{
    ChessMove, ChessMoveHint, Game, GameEnding, GameExplorer, LanMove, MoveOrdering, ParseFenError,
};
pub use primitives::{
    BitBoard, Board, CastlingRights, Color, File, NegDiag, Piece, PosDiag, Promotion, Rank, Square,
};
pub use search::{ParallelSearch, Score, SearchResult};

mod game;
mod primitives;
mod search;

#[cfg(test)]
mod perft;

/*
    TO BENCHMARK:
    - bitboard representation (classical / denser)
    - inlining (???)
*/
