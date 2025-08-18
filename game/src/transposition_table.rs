use position::PositionHash;
use std::{array, mem};

/// [Transposition table]
///
/// [Transposition table]: https://www.chessprogramming.org/Transposition_Table
pub struct TranspositionTable<const N: usize, V> {
    values: [Slot<V>; N],
}

impl<const N: usize, V> Default for TranspositionTable<N, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize, V> TranspositionTable<N, V> {
    #[must_use]
    pub fn new() -> Self {
        TranspositionTable {
            values: array::from_fn(|_| None),
        }
    }
}

pub type CollisionResult<'a, V> = Result<Entry<'a, V>, EntryCollision<'a, V>>;

impl<const N: usize, V> TranspositionTable<N, V> {
    pub fn entry<'a>(&'a mut self, hash: PositionHash) -> CollisionResult<'a, V> {
        let slot = self.slot_mut(hash);
        match slot.is_some() {
            true => {
                let entry = OccupiedEntry(slot);
                match *entry.key() == hash {
                    true => Ok(Entry::Occupied(entry)),
                    false => Err(EntryCollision { key: hash, entry }),
                }
            }
            false => Ok(Entry::Vacant(VacantEntry { key: hash, slot })),
        }
    }
    fn slot_mut(&mut self, hash: PositionHash) -> &mut Slot<V> {
        &mut self.values[hash % N]
    }
}

#[derive(Debug)]
pub struct EntryCollision<'a, V> {
    key: PositionHash,
    entry: OccupiedEntry<'a, V>,
}

impl<'a, V> EntryCollision<'a, V> {
    pub fn new_key(&self) -> &PositionHash {
        &self.key
    }
    pub fn old_key(&self) -> &PositionHash {
        self.entry.key()
    }
    pub fn entry_ref(&self) -> &OccupiedEntry<'a, V> {
        &self.entry
    }
    pub fn entry_mut(&mut self) -> &mut OccupiedEntry<'a, V> {
        &mut self.entry
    }
    // Should this also insert the new value?
    pub fn replace_key(self) -> OccupiedEntry<'a, V> {
        self.entry.0.as_mut().unwrap().0 = self.key;
        self.entry
    }
    pub fn keep_key(self) -> OccupiedEntry<'a, V> {
        self.entry
    }
}

#[derive(Debug)]
pub enum Entry<'a, V> {
    Occupied(OccupiedEntry<'a, V>),
    Vacant(VacantEntry<'a, V>),
}

impl<'a, V> Entry<'a, V> {
    pub fn or_insert(self, default: V) -> &'a mut V {
        self.or_insert_with(|| default)
    }
    pub fn or_insert_with<F>(self, default: F) -> &'a mut V
    where
        F: FnOnce() -> V,
    {
        self.or_insert_with_key(|_| default())
    }
    pub fn or_insert_with_key<F>(self, default: F) -> &'a mut V
    where
        F: FnOnce(&PositionHash) -> V,
    {
        match self {
            Entry::Occupied(occupied) => occupied.into_mut(),
            Entry::Vacant(vacant) => {
                let key = *vacant.key();
                vacant.insert(default(&key))
            }
        }
    }
    pub fn key(&self) -> &PositionHash {
        match self {
            Entry::Occupied(occupied) => occupied.key(),
            Entry::Vacant(vacant) => vacant.key(),
        }
    }
    pub fn and_modify<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut V),
    {
        if let Entry::Occupied(occupied) = &mut self {
            f(occupied.get_mut());
        }
        self
    }
    pub fn insert_entry(self, value: V) -> OccupiedEntry<'a, V> {
        let key = *self.key();
        let slot = match self {
            Entry::Occupied(occupied) => occupied.0,
            Entry::Vacant(vacant) => vacant.slot,
        };
        *slot = Some((key, value));
        OccupiedEntry(slot)
    }
}

impl<'a, V: Default> Entry<'a, V> {
    pub fn or_default(self) -> &'a mut V {
        self.or_insert_with(Default::default)
    }
}

#[derive(Debug)]
pub struct OccupiedEntry<'a, V>(&'a mut Slot<V>);

impl<'a, V> OccupiedEntry<'a, V> {
    pub fn key(&self) -> &PositionHash {
        &self.0.as_ref().unwrap().0
    }
    pub fn remove_entry(self) -> (PositionHash, V) {
        self.0.take().unwrap()
    }
    pub fn get(&self) -> &V {
        &self.0.as_ref().unwrap().1
    }
    pub fn get_mut(&mut self) -> &mut V {
        &mut self.0.as_mut().unwrap().1
    }
    pub fn into_mut(self) -> &'a mut V {
        &mut self.0.as_mut().unwrap().1
    }
    pub fn insert(&mut self, value: V) -> V {
        mem::replace(self.get_mut(), value)
    }
    pub fn remove(self) -> V {
        self.remove_entry().1
    }
}

#[derive(Debug)]
pub struct VacantEntry<'a, V> {
    key: PositionHash,
    slot: &'a mut Slot<V>,
}

impl<'a, V> VacantEntry<'a, V> {
    pub fn key(&self) -> &PositionHash {
        &self.key
    }
    pub fn into_key(self) -> PositionHash {
        self.key
    }
    pub fn insert(self, value: V) -> &'a mut V {
        self.insert_entry(value).into_mut()
    }
    pub fn insert_entry(self, value: V) -> OccupiedEntry<'a, V> {
        let internal = self.slot;
        *internal = Some((self.key, value));
        OccupiedEntry(internal)
    }
}

type Slot<V> = Option<(PositionHash, V)>;
