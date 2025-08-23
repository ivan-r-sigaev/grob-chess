use std::num::NonZeroU64;

use board::{Board, Color, File};
use either::Either;

use crate::position::{ChessMove, ChessUnmove, LanMove, Position};
use crate::raw_position::ParseFenError;
use crate::{CastlingRights, MoveList};

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
    /// Returns the move index when the 50 move rule counter was last reset.
    #[inline(always)]
    #[must_use]
    pub fn move_index_rule_50(&self) -> u32 {
        self.position.move_index_rule_50()
    }
    /// Returns the current move index.
    #[inline(always)]
    #[must_use]
    pub fn move_index(&self) -> u32 {
        self.position.move_index()
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
}

impl Game {
    /// Returns whether a given chess move is at least pseudo-legal in this position.
    pub fn is_move_pseudo_legal(&self, chess_move: ChessMove) -> bool {
        self.position.is_move_pseudo_legal(chess_move)
    }
    /// Returns an equivalent `ChessMove` for a `LanMove` in this position.
    pub fn lan_move(&self, lan_move: LanMove) -> Option<ChessMove> {
        self.position.lan_move(lan_move)
    }
    /// Make a [`ChessMove`] if it's valid.
    #[must_use]
    pub fn try_make_move(&mut self, chess_move: ChessMove) -> bool {
        if !self.is_move_pseudo_legal(chess_move) {
            return false;
        }
        self.make_move_unchecked(chess_move)
    }
    /// Unroll the last made move unless it's already the starting position.
    #[must_use]
    pub fn try_unmake_move(&mut self) -> bool {
        if self.is_history_empty() {
            return false;
        }
        self.unmake_move_unchecked();
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
    /// Violating the preconditions will corrupt the position's state.
    #[must_use]
    fn make_move_unchecked(&mut self, chess_move: ChessMove) -> bool {
        let hash = self.zobrist();
        let unmove = self.position.make_move(chess_move);
        if self.was_check_ignored() {
            self.position.unmake_move(unmove);
            return false;
        }

        self.history.push(PlyHistory { unmove, hash });
        true
    }
    /// Unroll the last made move unless it's already the starting position.
    ///
    /// # Panics
    /// Panics if there are no moves in the game's history.
    fn unmake_move_unchecked(&mut self) {
        let ply = self.history.pop().unwrap();
        self.position.unmake_move(ply.unmove);
    }
}

/// Expores through [`Game`]'s moves.
///
/// Even though [`GameExplorer`] holds a mutable reference
/// to the [`Game`], it will restore its original state
/// once the search is over.
#[derive(Debug)]
pub struct GameExplorer<'a> {
    game: &'a mut Game,
    move_list: MoveList,
}

/// Possible ending for a chess game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameEnding {
    /// Position ends in a draw.
    Stalemate,
    /// One of the players wins by checkmate.
    Checkmate,
}

impl GameExplorer<'_> {
    /// State of the game at this at this point during search.
    pub fn game(&self) -> &Game {
        self.game
    }
    /// Returns any legal move for this position or a [`GameEnding`]
    /// if the position has no legal moves.
    pub fn check_ending(&mut self) -> Either<ChessMove, GameEnding> {
        let mut any_move = None;
        let ending = self.for_each_legal_child_node(|node, chess_move| {
            any_move = Some(chess_move);
            node.exhaust_moves();
        });
        match ending {
            Some(ending) => Either::Right(ending),
            None => Either::Left(any_move.unwrap()),
        }
    }
    /// Makes a move and inspects the resulting state of the game
    /// with a function if the move is legal.
    /// Returns `true` if the move was legal.
    #[inline(always)]
    pub fn map_move_if_legal<F>(&mut self, chess_move: ChessMove, op: F) -> bool
    where
        F: FnOnce(&mut Self),
    {
        if !self.game.try_make_move(chess_move) {
            return false;
        }
        op(self);
        self.game.unmake_move_unchecked();
        true
    }
    /// Inspects all legal moves in position with a function.
    /// Returns `Some(game_ending: GameEnding)` if there are no legal moves.
    #[inline(always)]
    pub fn for_each_legal_child_node<F>(&mut self, mut op: F) -> Option<GameEnding>
    where
        F: FnMut(&mut Self, ChessMove),
    {
        self.move_list.push_group();
        self.game.position.push_moves(&mut |chess_move| {
            self.move_list.push_move(chess_move);
        });

        let mut has_moves = false;
        while let Some(chess_move) = self.move_list.pop_move() {
            if self.game.make_move_unchecked(chess_move) {
                has_moves = true;
                op(self, chess_move);
                self.game.unmake_move_unchecked();
            }
        }

        self.move_list.pop_group();

        if has_moves {
            None
        } else if self.game.is_check() {
            Some(GameEnding::Checkmate)
        } else {
            Some(GameEnding::Stalemate)
        }
    }
    /// Skips all next moves if currently
    /// inspecting legal moves using [`Self::for_each_legal_child_node`].
    ///
    /// # Panics
    /// Panics if not currently
    /// inspecting legal moves using [`Self::for_each_legal_child_node`].
    pub fn exhaust_moves(&mut self) {
        loop {
            if self.move_list.pop_move().is_none() {
                break;
            }
        }
    }
}

impl<'a> GameExplorer<'a> {
    fn new(game: &'a mut Game) -> Self {
        Self {
            game,
            move_list: MoveList::empty(),
        }
    }
}
