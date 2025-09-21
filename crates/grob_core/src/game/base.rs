use std::num::NonZeroU64;

use crate::game::position::{ChessUnmove, Position};
use crate::{Board, CastlingRights, ChessMove, Color, File, GameExplorer, LanMove, ParseFenError};

/// A chess game.
#[derive(Debug, Clone)]
pub struct Game {
    position: Position,
    history: Vec<PlyHistory>,
}

#[derive(Debug, Clone, Copy)]
struct PlyHistory {
    pub hash: NonZeroU64,
    pub unmove: ChessUnmove,
}

impl Game {
    /// Returns the initial position for a normal chess game.
    pub fn initial_position() -> Self {
        const INITIAL_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        Self::try_from_fen(INITIAL_FEN).unwrap()
    }

    /// Tries to parse a new game from [FEN].
    ///
    /// [fen]: https://www.chessprogramming.org/Forsyth-Edwards_Notation
    pub fn try_from_fen(fen: &str) -> Result<Self, ParseFenError> {
        Ok(Self {
            position: Position::try_from_fen(fen)?,
            history: Vec::new(),
        })
    }
    /// Returns a hash for the current position.
    #[inline(always)]
    #[must_use]
    pub fn zobrist(&self) -> NonZeroU64 {
        self.position.zobrist()
    }
    /// Returns the possible en passant target file if available or `None`.
    #[inline(always)]
    #[must_use]
    pub fn en_passant(&self) -> Option<File> {
        self.position.en_passant()
    }
    /// Returns the state of castling rights.
    #[inline(always)]
    #[must_use]
    pub fn castling_rights(&self) -> CastlingRights {
        self.position.castling_rights()
    }
    /// Returns a reference to the position's board.
    #[inline(always)]
    #[must_use]
    pub fn board(&self) -> &Board {
        self.position.board()
    }
    /// Returns the color of the player who is about to make a turn.
    #[inline(always)]
    #[must_use]
    pub fn turn(&self) -> Color {
        self.position.turn()
    }
    /// Returns the current state of halfmove clock.
    #[inline(always)]
    #[must_use]
    pub fn halfmove_clock(&self) -> u32 {
        self.position.halfmove_clock()
    }
    /// Returns the current move index.
    #[inline(always)]
    #[must_use]
    pub fn move_index(&self) -> u32 {
        self.position.ply_index()
    }
    /// Returns whether the king of the playing player is currently in check.
    pub fn is_check(&self) -> bool {
        self.board().is_king_in_check(self.turn())
    }
    /// Returns whether the king of the opponent player is currently in check.
    pub fn was_check_ignored(&self) -> bool {
        self.board().is_king_in_check(!self.turn())
    }
    /// Returns the number of times this position was played before in the game.
    pub fn count_repetitions(&self) -> usize {
        let hash = self.zobrist();
        self.history.iter().filter(|&ply| ply.hash == hash).count()
    }
    /// Returns `true` if this is the starting position for the game.
    pub fn is_history_empty(&self) -> bool {
        self.history.is_empty()
    }
    /// Returns whether a given chess move is at least pseudo-legal in this position.
    pub fn is_move_pseudo_legal(&self, chess_move: ChessMove) -> bool {
        self.position.is_move_pseudo_legal(chess_move)
    }
    /// Returns an equivalent `ChessMove` for a `LanMove` in this position.
    pub fn lan_move(&self, lan_move: LanMove) -> Option<ChessMove> {
        self.position.lan_move(lan_move)
    }
    /// Makes a [`ChessMove`].
    ///
    /// # Panics
    /// Panics if [`ChessMove`] is not legal.
    pub fn make_move(&mut self, chess_move: ChessMove) {
        let is_legal = self.try_make_move(chess_move);
        assert!(is_legal, "Attempted to make an illegal move!");
    }
    /// Unroll the last made move.
    ///
    /// # Panics
    /// Panics if there are no previous positions for this game.
    pub fn unmake_move(&mut self) {
        let is_legal = self.try_unmake_move();
        assert!(
            is_legal,
            "Attempted to unroll the game past the starting position!"
        )
    }
    /// Make a [`ChessMove`] if it's valid.
    #[must_use]
    pub fn try_make_move(&mut self, chess_move: ChessMove) -> bool {
        if !self.is_move_pseudo_legal(chess_move) {
            return false;
        }
        self.make_move_unchecked(chess_move)
    }
    /// Unroll the last made move unless there are no previous positions for this game.
    #[must_use]
    pub fn try_unmake_move(&mut self) -> bool {
        if self.is_history_empty() {
            return false;
        }
        self.position
            .unmake_move(self.history.pop().unwrap().unmove);
        true
    }
    /// Returns an explorer for this game.
    pub fn explore<'a>(&'a mut self) -> GameExplorer<'a> {
        GameExplorer::new(self)
    }
    /// Make a [`ChessMove`].
    ///
    /// # Preconditions
    /// - [`ChessMove`] must be at least pseudo-legal.
    ///
    /// Violating the preconditions will corrupt the game's state.
    #[must_use]
    pub(super) fn make_move_unchecked(&mut self, chess_move: ChessMove) -> bool {
        let hash = self.zobrist();
        let unmove = self.position.make_move(chess_move);
        if self.was_check_ignored() {
            self.position.unmake_move(unmove);
            return false;
        }

        self.history.push(PlyHistory { unmove, hash });
        true
    }
    /// Generates pseudo-legal moves for this position.
    pub(super) fn push_moves(&self, push_move: &mut impl FnMut(ChessMove)) {
        self.position.push_moves(push_move);
    }
}
