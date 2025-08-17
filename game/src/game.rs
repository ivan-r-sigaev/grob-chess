use crate::move_list::MoveList;
use position::position::{ChessMove, ChessUnmove, PositionHash, ParseFenError, Position};

#[derive(Debug, Clone)]
pub struct Game {
    pos: Position,
    move_list: MoveList,
    history: Vec<PlyHistory>,
}

#[derive(Debug, Clone, Copy)]
struct PlyHistory {
    hash: PositionHash,
    unmove: ChessUnmove,
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
            move_list: MoveList::empty(),
            history: Vec::new(),
        })
    }
    #[inline(always)]
    #[must_use]
    pub fn get_position(&self) -> &Position {
        &self.pos
    }
    pub fn count_repetitions(&self, hash: PositionHash) -> usize {
        self.history.iter().filter(|&ply| ply.hash == hash).count()
    }
    #[inline(always)]
    pub fn map_move_if_legal<F>(&mut self, move_concept: ChessMove, mut op: F) -> bool
    where
        F: FnMut(&mut Self, ChessMove),
    {
        if !self.pos.is_move_applicable(move_concept) {
            return false;
        }

        let unmove = self.pos.make_move(move_concept);
        let is_legal = !self.pos.was_check_ignored();
        if is_legal {
            op(self, move_concept);
        }
        self.pos.unmake_move(unmove);

        is_legal
    }
    #[inline(always)]
    pub fn for_each_legal_child_node<F>(&mut self, mut op: F) -> Option<GameEnding>
    where
        F: FnMut(&mut Self, ChessMove),
    {
        self.move_list.generate_moves(&self.pos);

        let mut has_moves = false;
        while let Some(next_move) = self.move_list.pop_move() {
            let unmove = self.pos.make_move(next_move);

            if !self.pos.was_check_ignored() {
                has_moves = true;
                op(self, next_move);
            }

            self.pos.unmake_move(unmove);
        }

        self.move_list.pop_group();

        if has_moves {
            None
        } else if self.pos.is_check() {
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
    pub fn try_make_move(mut self, move_concept: ChessMove) -> (Self, bool) {
        if !self.pos.is_move_applicable(move_concept) {
            return (self, false);
        }
        let hash = self.pos.position_hash();
        let unmove = self.pos.make_move(move_concept);
        if self.pos.was_check_ignored() {
            self.pos.unmake_move(unmove);
            return (self, false);
        }

        self.history.push(PlyHistory { unmove, hash });

        (self, true)
    }
    #[must_use]
    pub fn try_unmake_move(mut self) -> (Self, bool) {
        if let Some(ply_history) = self.history.pop() {
            self.pos.unmake_move(ply_history.unmove);
            return (self, true);
        }
        (self, false)
    }
}
