use position::position::PositionHash;

pub struct TranspositionTable<const N: usize, V> {
    values: [Option<(PositionHash, V)>; N],
}

impl<const N: usize, V: Copy> Default for TranspositionTable<N, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize, V: Copy> TranspositionTable<N, V> {
    #[must_use]
    pub fn new() -> Self {
        TranspositionTable { values: [None; N] }
    }
}

impl<const N: usize, V> TranspositionTable<N, V> {
    pub fn get(&self, hash: PositionHash) -> Option<&V> {
        self.values[hash % N]
            .as_ref()
            .filter(|(key, _)| *key == hash)
            .map(|(_, value)| value)
    }
    pub fn get_mut(&mut self, hash: PositionHash) -> Option<&mut V> {
        self.values[hash % N]
            .as_mut()
            .filter(|(key, _)| *key == hash)
            .map(|(_, value)| value)
    }
    pub fn insert(&mut self, hash: PositionHash, value: V) {
        self.values[hash % N] = Some((hash, value));
    }
}
