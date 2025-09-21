use crate::{game::PackedChessMove, ChessMove};

/// Stores chess moves during search.
#[derive(Debug, Clone)]
pub struct MoveList {
    moves: Vec<PackedChessMove>,
    lens: Vec<usize>,
}

impl MoveList {
    /// Constructs a new [`MoveList`].
    #[inline(always)]
    #[must_use]
    pub fn empty() -> MoveList {
        MoveList {
            moves: Vec::new(),
            lens: Vec::new(),
        }
    }
    /// Returns a mutable slice to the current move group.
    ///
    /// # Panics
    /// Panics if there is no current move group.
    pub fn group_mut(&mut self) -> &mut [PackedChessMove] {
        let end = self.moves.len();
        let start = end - self.group_len();
        &mut self.moves[start..end]
    }
    /// Removes and returns a move from end of the current move group.
    ///
    /// # Panics
    /// Panics if there is no current move group.
    #[inline(always)]
    #[must_use]
    pub fn pop_move(&mut self) -> Option<ChessMove> {
        if self.group_len() == 0 {
            return None;
        }
        *self.group_len_mut() -= 1;
        self.moves.pop().map(|packed| packed.get())
    }
    /// Pushes a move into the end of the current move group.
    ///
    /// # Panics
    /// Panics if there is no current move group.
    #[inline(always)]
    pub fn push_move(&mut self, chess_move: ChessMove) {
        self.moves.push(PackedChessMove::new(chess_move));
        *self.group_len_mut() += 1;
    }
    /// Discard current move group and restores the old move group (if any).
    ///
    /// # Panics
    /// Panics if there is no current move group.
    #[inline(always)]
    pub fn pop_group(&mut self) {
        let len = self.lens.pop().expect("move list has no more groups");
        self.moves.truncate(self.moves.len() - len);
    }
    /// Saves the old move group (if any) and starts a new one (current).
    #[inline(always)]
    pub fn push_group(&mut self) {
        self.lens.push(0);
    }
    #[inline(always)]
    #[must_use]
    fn group_len(&self) -> usize {
        *self.lens.last().unwrap()
    }
    #[inline(always)]
    #[must_use]
    fn group_len_mut(&mut self) -> &mut usize {
        self.lens.last_mut().unwrap()
    }
}
