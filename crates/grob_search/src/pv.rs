use grob_core::game::movegen::PackedChessMove;

use crate::ChessMove;

/// Table that is used to collect the [Principal Variation] during the search.
///
/// [Principal Variation]: https://www.chessprogramming.org/Principal_Variation
#[derive(Debug, Clone)]
pub struct PVTable {
    pv_capacity: usize,
    pv_len: usize,
    data: Box<[Option<PackedChessMove>]>,
}

impl PVTable {
    /// Creates a new table that can hold a PV up to a specified number of plies (moves).
    pub fn new(pv_capacity: usize) -> Self {
        let pv_len = 0;
        let data = vec![None; trnum(pv_capacity)].into_boxed_slice();
        Self {
            pv_capacity,
            pv_len,
            data,
        }
    }
    /// Return the maximum PV length (in plies), that this table can hold.
    pub fn pv_capacity(&self) -> usize {
        self.pv_capacity
    }
    /// Returns the current length (in plies) of the PV stored in the table.
    pub fn pv_len(&self) -> usize {
        self.pv_len
    }
    /// Stores the PV move for the specified depth.
    ///
    /// # Panics
    /// - Panics if `depth` is not strictly less than the PV capacity.
    /// - Panics if `depth` is greater than the current PV length.
    pub fn store(&mut self, chess_move: ChessMove, depth: usize) {
        assert!(
            depth < self.pv_capacity,
            "The PV table is not big enough to store the PV at the specified depth!"
        );
        assert!(
            depth <= self.pv_len,
            "To store the PV move at depth N != 0, one must first store the PV move at depth N-1!"
        );
        if depth == self.pv_len {
            self.pv_len += 1;
        }
        let start = trnum(depth);
        self.data[start] = Some(PackedChessMove::new(chess_move));
        for i in 0..depth {
            self.data[start + 1 + i] = self.data[start - depth + i];
        }
    }
    /// Returns an iterator to the largest PV that is currently stored in the table.
    pub fn pv(&self) -> PVIter<'_> {
        let start = trnum(self.pv_len.saturating_sub(1));
        let end = start + self.pv_len;
        let inner = self.data[start..end].iter();
        PVIter { inner }
    }
    /// Forgets all PVs that are currently stored in the table and sets the PV length to zero.
    pub fn clear(&mut self) {
        self.pv_len = 0;
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
