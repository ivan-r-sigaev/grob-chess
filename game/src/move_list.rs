use position::position::{ChessMove, PackedChessMove, Position};

#[derive(Debug, Clone)]
pub struct MoveList {
    moves: Vec<PackedChessMove>,
    lens: Vec<usize>,
    len: usize,
}

impl MoveList {
    #[inline(always)]
    #[must_use]
    pub fn empty() -> MoveList {
        MoveList {
            moves: Vec::new(),
            lens: Vec::new(),
            len: 0,
        }
    }
    #[inline(always)]
    pub fn generate_moves(&mut self, position: &Position) {
        // TODO: could optimize for double checks here...
        self.push_group();
        let push_move = &mut |chess_move| {
            self.push_move(chess_move);
        };
        position.push_pawn_attacks(push_move);
        position.push_knight_attacks(push_move);
        position.push_bishop_attacks(push_move);
        position.push_rook_attacks(push_move);
        position.push_king_attacks(push_move);

        position.push_castlings(push_move);
        position.push_king_quiets(push_move);
        position.push_rook_quiets(push_move);
        position.push_bishop_quiets(push_move);
        position.push_knight_quiets(push_move);
        position.push_pawn_quiets(push_move);
    }
    #[inline(always)]
    #[must_use]
    pub fn pop_move(&mut self) -> Option<ChessMove> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        self.moves.pop().map(|packed| packed.get())
    }
    #[inline(always)]
    pub fn pop_group(&mut self) {
        self.moves.truncate(self.moves.len() - self.len);
        self.len = self.lens.pop().expect("move list has no more groups");
    }
    #[inline(always)]
    pub fn clear(&mut self) {
        self.moves.clear();
        self.lens.clear();
        self.len = 0;
    }
    #[inline(always)]
    fn push_move(&mut self, chess_move: ChessMove) {
        self.moves.push(PackedChessMove::new(chess_move));
        self.len += 1;
    }
    #[inline(always)]
    fn push_group(&mut self) {
        self.lens.push(self.len);
        self.len = 0;
    }
}

#[cfg(test)]
mod tests;
