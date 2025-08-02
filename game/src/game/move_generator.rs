use position::prelude::{ChessMove, ChessMoveHint, Position, Square};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PackedChessMove {
    data: u16,
}

impl PackedChessMove {
    #[inline(always)]
    #[must_use]
    pub fn new(chess_move: ChessMove) -> Self {
        Self {
            data: (((chess_move.hint as u16) & 0xf) << 12)
                | (((chess_move.from as u16) & 0x3f) << 6)
                | ((chess_move.to as u16) & 0x3f),
        }
    }
    #[inline(always)]
    #[must_use]
    pub fn get(self) -> ChessMove {
        let to = Square::from_repr((self.data & 0x3f) as u8).unwrap();
        let from = Square::from_repr(((self.data >> 6) & 0x3f) as u8).unwrap();
        let hint = ChessMoveHint::from_repr(((self.data >> 12) & 0x0f) as u8).unwrap();
        ChessMove { to, from, hint }
    }
}

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
        position.push_king_moves(push_move);
        position.push_knight_moves(push_move);
        position.push_bishop_moves(push_move);
        position.push_rook_moves(push_move);
        position.push_pawn_attacks(push_move);
        position.push_pawn_quiets(push_move);
        position.push_castlings(push_move);
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
