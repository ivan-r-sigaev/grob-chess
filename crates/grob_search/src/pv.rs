use grob_core::game::movegen::PackedChessMove;

use crate::ChessMove;

/// Table that is used to collect the [Principal Variation] during the search.
///
/// [Principal Variation]: https://www.chessprogramming.org/Principal_Variation
#[derive(Debug, Clone)]
pub struct PvTable {
    ply_capacity: usize,
    ply_len: usize,
    data: Box<[Option<PackedChessMove>]>,
}

impl PvTable {
    /// Creates a new table that can hold a PV up to a specified number of plies (moves).
    pub fn new(ply_capacity: usize) -> Self {
        let ply_len = 0;
        let data = vec![None; trnum(ply_capacity)].into_boxed_slice();
        Self {
            ply_capacity,
            ply_len,
            data,
        }
    }
    /// Return the maximum PV length (in plies), that this table can hold.
    pub fn pv_capacity(&self) -> usize {
        self.ply_capacity
    }
    /// Returns the current length (in plies) of the PV stored in the table.
    pub fn pv_len(&self) -> usize {
        self.ply_len
    }
    /// Copies the other PV table at the specified ply.
    /// 
    /// # Panics
    /// - Panics if the table's length is smaller than the specified ply.
    /// - Panics if the table's lengths are not the same.
    pub fn copy_ply(&mut self, other: &Self, ply: usize) {
        assert!(self.ply_capacity == other.ply_capacity, "The tables should have the same length!");
        assert!(ply < other.ply_len, "The other PV table is too small!");
        let depth = (self.ply_capacity - 1) - ply;
        let start = trnum(depth);
        for i in 0..(depth - (self.ply_capacity - self.ply_len) + 1) {
            self.data[start + i] = other.data[start + i];
        }
    }
    /// Stores the PV move for the specified ply.
    ///
    /// # Panics
    /// - Panics if `ply` is not strictly less than the PV capacity.
    /// - Panics if `ply` is greater than the current PV length.
    pub fn store(&mut self, chess_move: ChessMove, ply: usize) {
        assert!(
            ply < self.ply_capacity,
            "The PV table is not big enough to store the PV at the specified ply!"
        );
        assert!(
            ply <= self.ply_len,
            "To store the PV move at ply N != 0, one must first store the PV move at ply N-1!"
        );
        if ply == self.ply_len {
            self.ply_len += 1;
        }
        let depth = (self.ply_capacity - 1) - ply;
        let start = trnum(depth);
        self.data[start] = Some(PackedChessMove::new(chess_move));
        for i in 0..(depth - (self.ply_capacity - self.ply_len)) {
            self.data[start + 1 + i] = self.data[start - depth + i];
        }
    }
    /// Returns an iterator to the largest PV that is currently stored in the table.
    pub fn pv(&self) -> PVIter<'_> {
        let start = trnum(self.ply_len.saturating_sub(1));
        let end = start + self.ply_len;
        let inner = self.data[start..end].iter();
        PVIter { inner }
    }
    /// Forgets PVs that are stored in the table with the ply index
    /// greater than or equal to the specified one.
    pub fn clear(&mut self, ply: usize) {
        self.ply_len = ply;
    }
    /// Makes one step "into" the PV, reducing the overall ply length by one.
    pub fn apply_pv_move(&mut self) {
        for ply in 0..self.ply_len {
            let start = trnum(ply);
            let len = self.ply_len - ply;
            for i in 0..(len - 1) {
                self.data[start + i] = self.data[start + i + 1];
            }
        }
        self.ply_len = self.ply_len.saturating_sub(1);
    }
}

/// Helper iterator to access the PV stored in the [`PVTable`].
pub struct PVIter<'a> {
    inner: std::slice::Iter<'a, Option<PackedChessMove>>,
}

impl Iterator for PVIter<'_> {
    type Item = ChessMove;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|opt| opt.unwrap().get())
    }
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.inner.nth(n).map(|opt| opt.unwrap().get())
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

/// Returns the `n`th triangular number.
const fn trnum(n: usize) -> usize {
    n * (n + 1) / 2
}
