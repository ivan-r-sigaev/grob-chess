use std::fmt;

use parking_lot::RwLock;
use position::{ChessMove, PositionHash};

use crate::{KeyLookup, WeakHashMap};

/// A [transposition]. 
/// 
/// Designed to be stored in the [transposition table].
/// 
/// [transposition table]: https://www.chessprogramming.org/Transposition_Table
/// [transposition]: https://www.chessprogramming.org/Transposition
#[derive(Debug, Clone, Copy)]
pub struct Transposition {
    /// Best move on the basis of past search.
    pub best_move: ChessMove,
}

/// A [transposition table].
/// 
/// Transposition table uses an `RwLock` internally so that
/// it can safely be shared between threads.
///
/// [transposition table]: https://www.chessprogramming.org/Transposition_Table
pub struct TranspositionTable(RwLock<WeakHashMap<Transposition>>);

impl TranspositionTable {
    /// Constructs a [`TranspositionTable`] that can hold
    /// a specified number of transpositions.
    ///
    /// # Panics
    /// - Panics if `capacity` is zero.
    /// - Panics if `capacity * size_of::<Transposition>()` exceeds `isize::MAX`.
    pub fn new(capacity: usize) -> Self {
        Self(RwLock::new(WeakHashMap::new(capacity)))
    }
    /// Returns the maximum number of [`Transposition`]s this 
    /// table can hold at the same time.
    pub fn capacity(&self) -> usize {
        self.0.read().capacity()
    }
    /// Returns the [`Transposition`] with the exactly matching hash 
    /// or `None` if one is not available.
    pub fn get_exact(&self, hash: PositionHash) -> Option<Transposition> {
        self.0.read().get(hash.get()).exact().map(|(_, value)| *value)
    }
    /// Returns any [`Transposition`] that matches the hash 
    /// or `None` if no transpositions match.
    /// 
    /// The result may be type-2 hash collision.
    pub fn get(&self, hash: PositionHash) -> Option<Transposition> {
        match self.0.read().get(hash.get()) {
            KeyLookup::Exact(_, res) => Some(*res),
            KeyLookup::Clash(_, res) => Some(*res),
            KeyLookup::Empty => None,
        }
    }
    /// Saves the [`Transposition`] to the table.
    /// 
    /// This will overwrite the [`Transposition`] with the 
    /// clashing hash if one exists.
    pub fn insert(&self, hash: PositionHash, value: Transposition) {
        _ = self.0.write().entry(hash.get()).insert(value);
    }
}

impl fmt::Debug for TranspositionTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("TranspositionTable").finish()
    }
}
