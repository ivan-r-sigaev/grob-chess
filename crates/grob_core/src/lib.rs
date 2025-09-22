//! Position
//!
//! This crate provides types related to game representation and move generation.

pub use primitives::{
    BitBoard, Board, CastlingRights, Color, File, NegDiag, Piece, PosDiag, Promotion, Rank, Square,
};
pub use search::{
    spawn_search_server, Score, SearchRequest, SearchResult, ServerCommand, ServerResponse,
};

pub use game::{
    ChessMove, ChessMoveHint, Game, GameEnding, GameTreeWalker, LanMove, MoveOrdering,
    PackedChessMove, ParseFenError,
};

mod game;
mod primitives;
mod search;

/*
    TO BENCHMARK:
    - bitboard representation (classical / denser)
    - inlining (???)
*/
