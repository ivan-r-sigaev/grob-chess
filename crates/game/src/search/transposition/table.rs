use std::{fmt, num::NonZeroU64};

use parking_lot::RwLock;

use crate::{search::transposition::table_base::TranspositionTableBase, ChessMove, Score};

/// A [transposition].
///
/// Designed to be stored in the [transposition table].
///
/// [transposition table]: https://www.chessprogramming.org/Transposition_Table
/// [transposition]: https://www.chessprogramming.org/Transposition
#[derive(Debug, Clone, Copy)]
pub struct Transposition {
    /// Best move on the basis of the past search.
    pub best_move: ChessMove,
    /// The [`Score`] of the position on the basis of the past search.
    pub score: Score,
    /// The depth of the past search.
    pub depth: u64, // TODO: is u64 too large?
}

/// A [transposition table].
///
/// Transposition table uses an `RwLock` internally so that
/// it can safely be shared between threads.
///
/// [transposition table]: https://www.chessprogramming.org/Transposition_Table
pub struct TranspositionTable(RwLock<TranspositionTableBase<Transposition>>);

impl TranspositionTable {
    /// Size of a single [`Transposition`] within the [`TranspositionTable`].
    pub const ITEM_SIZE: usize = TranspositionTableBase::<Transposition>::ITEM_SIZE;

    /// Constructs a [`TranspositionTable`] that can hold
    /// a specified number of transpositions.
    ///
    /// # Panics
    /// - Panics if `capacity` is zero.
    pub fn new(capacity: usize) -> Self {
        Self(RwLock::new(TranspositionTableBase::new(capacity)))
    }
    // /// Returns the maximum number of [`Transposition`]s this
    // /// table can hold at the same time.
    // pub fn capacity(&self) -> usize {
    //     self.0.read().capacity()
    // }
    /// Returns the [`Transposition`] with the exactly matching hash
    /// or `None` if one is not available.
    pub fn get(&self, hash: NonZeroU64) -> Option<Transposition> {
        self.0
            .read()
            .get(hash)
            .filter(|item| item.is_exact())
            .map(|item| *item)
    }
    /// Saves the [`Transposition`] to the table and returns the repaced [`Transposition`].
    ///
    /// This will overwrite the [`Transposition`] with the
    /// clashing hash if one exists.
    pub fn insert(&self, hash: NonZeroU64, value: Transposition) -> Option<Transposition> {
        self.0.write().insert(hash, value)
    }
    // /// Inserts the new value or replaces the old one if the predicate returns `true`.
    // pub fn insert_or_replace_if<P>(
    //     &self,
    //     hash: NonZeroU64,
    //     value: Transposition,
    //     pred: P,
    // ) -> Option<Transposition>
    // where
    //     P: FnOnce(&Transposition) -> bool,
    // {
    //     let mut read = self.0.upgradable_read();
    //     if read.get(hash).is_none_or(|item| pred(item.get())) {
    //         read.with_upgraded(|cache| cache.insert(hash, value))
    //     } else {
    //         None
    //     }
    // }
    /// Clears all saved [`Transposition`]s.
    pub fn clear(&self) {
        self.0.write().clear();
    }
    // /// Resize the transposition table.
    // ///
    // /// Calling this will also have the same effect as [`Self::clear`].
    // pub fn resize(&self, new_capacity: usize) {
    //     *self.0.write() = Cache::new(new_capacity);
    // }
}

impl fmt::Debug for TranspositionTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("TranspositionTable").finish()
    }
}
