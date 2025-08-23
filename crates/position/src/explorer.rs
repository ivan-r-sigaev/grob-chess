use either::Either;

use crate::{ChessMove, Game, MoveList};

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

impl<'a> GameExplorer<'a> {
    pub(crate) fn new(game: &'a mut Game) -> Self {
        Self {
            game,
            move_list: MoveList::empty(),
        }
    }
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
        self.game.push_moves(&mut |chess_move| {
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
