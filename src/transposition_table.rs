use crate::game::move_generator::position::PositionHash;

pub struct TranspositionTable<const N: usize, V: Copy> {
    values: [Option<(PositionHash, V)>; N]
}

impl<const N: usize, V: Copy> TranspositionTable<N, V> {
    pub fn new() -> Self {
        return TranspositionTable {
            values: [None; N],
        }
    }
}

impl<const N: usize, V: Copy> TranspositionTable<N, V> {
    pub fn get(&self, hash: PositionHash) -> Option<&V> {
        return match &self.values[hash % N] {
            Some(kvp) => if kvp.0 == hash { Some(&kvp.1) } else { None },
            None => None,
        }
    }
    pub fn get_mut(&mut self, hash: PositionHash) -> Option<&mut V> {
        return match &mut self.values[hash % N] {
            Some(kvp) => if kvp.0 == hash { Some(&mut kvp.1) } else { None },
            None => None,
        }
    }
    pub fn insert(&mut self, hash: PositionHash, value: V) {
        self.values[hash % N] = Some((hash, value));
    }
}
