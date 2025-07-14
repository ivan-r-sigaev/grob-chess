pub mod move_generator;

use move_generator::{
    can_make_move, make_move, unmake_move, MoveConcept, MoveGenerator, UnmoveConcept,
};
use position::prelude::{ParseFenError, Position};

#[derive(Debug, Clone)]
pub struct Game {
    pos: Position,
    move_list: MoveGenerator,
    history: Vec<PlyHistory>,
}

#[derive(Debug, Clone, Copy)]
struct PlyHistory {
    unmove: UnmoveConcept,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameEnding {
    Stalemate,
    Checkmate,
}

impl Game {
    #[inline(always)]
    pub fn try_from_fen(fen: &str) -> Result<Game, ParseFenError> {
        Ok(Game {
            pos: Position::try_from_fen(fen)?,
            move_list: MoveGenerator::empty(),
            history: Vec::new(),
        })
    }
    #[inline(always)]
    #[must_use]
    pub fn get_position(&self) -> &Position {
        &self.pos
    }
    #[inline(always)]
    pub fn map_move_if_legal<F>(&mut self, move_concept: MoveConcept, mut op: F) -> bool
    where
        F: FnMut(&mut Self, MoveConcept),
    {
        if !can_make_move(&self.pos, move_concept) {
            return false;
        }

        let unmove = make_move(&mut self.pos, move_concept);
        let is_legal = !self.pos.board().is_king_in_check(!self.pos.turn());
        if is_legal {
            op(self, move_concept);
        }
        unmake_move(&mut self.pos, unmove);

        is_legal
    }
    #[inline(always)]
    pub fn for_each_legal_child_node<F>(&mut self, mut op: F) -> Option<GameEnding>
    where
        F: FnMut(&mut Self, MoveConcept),
    {
        self.move_list.generate_moves(&self.pos);

        let mut has_moves = false;
        while let Some(next_move) = self.move_list.pop_move() {
            let unmove = make_move(&mut self.pos, next_move);

            if !self.pos.board().is_king_in_check(!self.pos.turn()) {
                has_moves = true;
                op(self, next_move);
            }

            unmake_move(&mut self.pos, unmove);
        }

        self.move_list.pop_group();

        if has_moves {
            None
        } else if self.pos.board().is_king_in_check(self.pos.turn()) {
            Some(GameEnding::Checkmate)
        } else {
            Some(GameEnding::Stalemate)
        }
    }
    #[inline(always)]
    pub fn exhaust_moves(&mut self) {
        loop {
            if self.move_list.pop_move().is_none() {
                break;
            }
        }
    }
    #[must_use]
    pub fn try_make_move(mut self, move_concept: MoveConcept) -> (Self, bool) {
        if !can_make_move(&self.pos, move_concept) {
            return (self, false);
        }

        let unmove = make_move(&mut self.pos, move_concept);
        if self.pos.board().is_king_in_check(!self.pos.turn()) {
            unmake_move(&mut self.pos, unmove);
            return (self, false);
        }

        // TODO: Also check for threefold repetition, etc.

        self.history.push(PlyHistory { unmove });

        (self, true)
    }
    #[must_use]
    pub fn try_unmake_move(mut self) -> (Self, bool) {
        if let Some(ply_history) = self.history.pop() {
            unmake_move(&mut self.pos, ply_history.unmove);
            return (self, true);
        }
        (self, false)
    }
}
