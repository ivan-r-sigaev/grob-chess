//! Position
//!
//! This crate provides types related to game representation and move generation.

pub use primitives::{
    BitBoard, Board, CastlingRights, Color, File, NegDiag, Piece, PosDiag, Promotion, Rank, Square,
};
pub use search::{
    spawn_search_server, Score, SearchRequest, SearchResult, ServerCommand, ServerResponse,
};

pub use game::{ChessMove, ChessMoveHint, Game, LanMove, ParseFenError};
pub use walker::{GameEnding, GameTreeWalker, MoveOrdering};

mod game;
mod primitives;
mod search;
mod walker;

#[cfg(test)]
mod perft;

/*
    TO BENCHMARK:
    - bitboard representation (classical / denser)
    - inlining (???)
*/
