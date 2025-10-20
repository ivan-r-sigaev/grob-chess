//! Position
//!
//! This crate provides types related to game representation and move generation.

pub use search::{spawn_search_server, SearchRequest, SearchResult, ServerCommand, ServerResponse};

pub use game::{
    ChessMove, ChessMoveHint, Game, GameEnding, GameTreeWalker, LanMove, MoveOrdering,
    PackedChessMove,
};

pub mod bitboard;
pub mod board;
pub mod castling_rights;
pub mod game;
pub mod pieces;
pub mod score;
pub mod search;
pub mod square;

mod position;

/*
    TO BENCHMARK:
    - bitboard representation (classical / denser)
    - inlining (???)
*/
