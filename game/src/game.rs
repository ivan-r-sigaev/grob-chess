use crate::move_list::MoveList;
use either::Either;
use position::position::{ChessMove, ChessUnmove, PositionHash, ParseFenError, Position};

#[derive(Debug, Clone)]
pub struct Game {
    pos: Position,
    history: Vec<PlyHistory>,
}

impl Game {
    #[inline(always)]
    pub fn try_from_fen(fen: &str) -> Result<Game, ParseFenError> {
        Ok(Game {
            pos: Position::try_from_fen(fen)?,
            history: Vec::new(),
        })
    }
    #[inline(always)]
    #[must_use]
    pub fn position(&self) -> &Position {
        &self.pos
    }
    pub fn count_repetitions(&self, hash: PositionHash) -> usize {
        self.history.iter().filter(|&ply| ply.hash == hash).count()
    }
    pub fn search(&mut self) -> GameSearch<'_> {
        todo!()
    }
    #[must_use]
    pub fn try_make_move(&mut self, chess_move: ChessMove) -> bool {
        if !self.pos.is_move_applicable(chess_move) {
            return false;
        }
        self.make_move(chess_move)
    }
    #[must_use]
    pub fn try_unmake_move(&mut self) -> bool {
        if self.history.is_empty() {
            return false;
        }
        self.unmake_move();
        true
    }
    #[must_use]
    fn make_move(&mut self, chess_move: ChessMove) -> bool {
        let hash = self.pos.position_hash();
        let unmove = self.pos.make_move(chess_move);
        if self.pos.was_check_ignored() {
            self.pos.unmake_move(unmove);
            return false;
        }

        self.history.push(PlyHistory { unmove, hash });
        false
    }
    fn unmake_move(&mut self) {
        let ply = self.history.pop().unwrap();
        self.pos.unmake_move(ply.unmove);
    }
}

#[derive(Debug)]
pub struct GameSearch<'a> {
    game: &'a mut Game,
    move_list: MoveList,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameEnding {
    Stalemate,
    Checkmate,
}

impl<'a> GameSearch<'a> {
    pub fn new(game: &'a mut Game) -> Self {
        Self {
            game,
            move_list: MoveList::empty(),
        }
    }
}

impl GameSearch<'_> {
    pub fn game(&self) -> &Game {
        self.game
    }
    pub fn check_ending(&mut self) -> Either<ChessMove, GameEnding> {
        let mut any_move = None;
        let ending = self.for_each_legal_child_node(|node, chess_move| {
            any_move = Some(chess_move);
            node.exhaust_moves();
        });
        match ending {
            Some(ending) => Either::Right(ending),
            None => Either::Left(any_move.unwrap())
        }
    }
    #[inline(always)]
    pub fn map_move_if_legal<F>(&mut self, chess_move: ChessMove, op: F) -> bool
    where
        F: FnOnce(&mut Self),
    {
        if !self.game.try_make_move(chess_move) {
            return false;
        }
        op(self);
        self.game.unmake_move();
        true
    }
    #[inline(always)]
    pub fn for_each_legal_child_node<F>(&mut self, mut op: F) -> Option<GameEnding>
    where
        F: FnMut(&mut Self, ChessMove),
    {
        self.move_list.generate_moves(self.game.position());

        let mut has_moves = false;
        while let Some(chess_move) = self.move_list.pop_move() {
            if self.game.make_move(chess_move) {
                has_moves = true;
                op(self, chess_move);
                self.game.unmake_move();
            }
        }

        self.move_list.pop_group();

        if has_moves {
            None
        } else if self.game.position().is_check() {
            Some(GameEnding::Checkmate)
        } else {
            Some(GameEnding::Stalemate)
        }
    }
    pub fn exhaust_moves(&mut self) {
        loop {
            if self.move_list.pop_move().is_none() {
                break;
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct PlyHistory {
    hash: PositionHash,
    unmove: ChessUnmove,
}
