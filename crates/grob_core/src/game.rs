use std::num::NonZeroU64;

use crate::{
    board::Board, castling_rights::CastlingRights, game::make::ChessUnmove, pieces::Color,
    position::Position, square::File,
};

pub mod fen;
pub mod lan;
pub mod make;
pub mod movegen;
pub mod walker;

/// A chess position.
#[derive(Debug, Clone)]
pub struct Game {
    history: Vec<PlyHistory>,
    position: Position,
    move_index_rule_50: u32,
    move_index: u32,
}

#[derive(Debug, Clone, Copy)]
struct PlyHistory {
    hash: NonZeroU64,
    unmove: ChessUnmove,
}

impl Game {
    /// Returns the initial position for the standard chess game.
    pub fn initial_position() -> Self {
        const INITIAL_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        Self::try_from_fen(INITIAL_FEN).unwrap()
    }
    /// Returns a hash for the current position.
    #[must_use]
    pub fn zobrist(&self) -> NonZeroU64 {
        self.position.zobrist()
    }
    /// Returns the possible en passant target file if available or `None`.
    #[must_use]
    pub fn en_passant(&self) -> Option<File> {
        self.position.en_passant()
    }
    /// Returns the state of castling rights.
    #[must_use]
    pub fn castling_rights(&self) -> CastlingRights {
        self.position.castling_rights()
    }
    /// Returns a reference to the position's board.
    #[must_use]
    pub fn board(&self) -> &Board {
        self.position.board()
    }
    /// Returns the color of the player who is about to make a turn.
    #[must_use]
    pub fn turn(&self) -> Color {
        self.position.turn()
    }
    /// Returns the current state of the halfmove clock.
    #[must_use]
    pub fn halfmove_clock(&self) -> u32 {
        self.move_index - self.move_index_rule_50
    }
    /// Returns the number of plies played so far.
    #[must_use]
    pub fn ply_index(&self) -> u32 {
        self.move_index
    }
    /// Returns the number of times this position was played before in the game.
    pub fn count_repetitions(&self) -> usize {
        let hash = self.zobrist();
        self.history.iter().filter(|&ply| ply.hash == hash).count()
    }
    /// Returns `true` if there are no moves to roll back.
    pub fn is_history_empty(&self) -> bool {
        self.history.is_empty()
    }

    /// Returns `true` if the king of the playing player is currently in check.
    pub fn is_check(&self) -> bool {
        self.board().is_king_in_check(self.turn())
    }
    /// Returns `true` if the positions are the same (50 move rule, move index, and history are ignored).
    pub fn is_same_position(&self, other: &Self) -> bool {
        self.position == other.position
    }
    /// Returns `true` if the king of the opponent player is currently in check.
    fn can_capture_king(&self) -> bool {
        self.board().is_king_in_check(!self.turn())
    }
    /// Increments the move index and returns the previous state of halfmove clock.
    fn next_move_index(&mut self, reset_hm_clock: bool) -> u32 {
        let res = self.move_index - self.move_index_rule_50;
        self.move_index += 1;
        if reset_hm_clock {
            self.move_index_rule_50 = self.move_index;
        }
        res
    }
    /// Decrements the move index and set the state of halfmove clock.
    fn prev_move_index(&mut self, hm_clock_state: u32) {
        self.move_index = self.move_index.strict_sub(1);
        self.move_index_rule_50 = self.move_index.strict_sub(hm_clock_state);
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::initial_position()
    }
}
