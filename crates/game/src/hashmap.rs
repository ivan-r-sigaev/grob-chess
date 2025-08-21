use std::{mem, num::NonZeroU64};

/// A fixed size collection simialr to [`std::collections::HashMap`],
/// that does not enforce uniqueness of its keys.
///
/// This collection is primarily useful when designing a [transposition table].
///
/// [transposition table]: https://www.chessprogramming.org/Transposition_Table
pub struct WeakHashMap<V>(Box<[Item<V>]>);

type Item<V> = Option<(NonZeroU64, V)>;

/// A entry into the [`WeakHashMap`].
#[derive(Debug)]
pub enum Entry<'a, V> {
    /// Entry's key matched exactly.
    Exact(ExactEntry<'a, V>),
    /// Entry's key clashed.
    Clash(ClashEntry<'a, V>),
    /// Entry's key was not found.
    Empty(EmptyEntry<'a, V>),
}

impl<'a, V> Entry<'a, V> {
    /// Inserts the new value with the new key regardless of the entry's state.
    pub fn insert(self, value: V) -> ExactEntry<'a, V> {
        match self {
            Entry::Exact(mut exact_entry) => {
                exact_entry.insert(value);
                exact_entry
            }
            Entry::Clash(clash_entry) => {
                let mut exact_entry = clash_entry.with_new_key();
                exact_entry.insert(value);
                exact_entry
            }
            Entry::Empty(empty_entry) => empty_entry.insert(value),
        }
    }
}

/// Entry which's key matched exactly.
#[derive(Debug)]
pub struct ExactEntry<'a, V> {
    item: &'a mut Item<V>,
}

impl<'a, V> ExactEntry<'a, V> {
    /// Returns the entry's currently used key.
    pub fn key(&self) -> NonZeroU64 {
        self.item.as_ref().unwrap().0
    }
    /// Returns a reference to the entry's value.
    pub fn get(&self) -> &V {
        &self.item.as_ref().unwrap().1
    }
    /// Returns a mutable reference to the entry's value.
    pub fn get_mut(&mut self) -> &mut V {
        &mut self.item.as_mut().unwrap().1
    }
    /// Converts the entry to a mutable reference to its value.
    pub fn into_mut(self) -> &'a mut V {
        &mut self.item.as_mut().unwrap().1
    }
    /// Inserts a new value into the entry and returns the old one.
    pub fn insert(&mut self, value: V) -> V {
        mem::replace(self.get_mut(), value)
    }
    /// Removes the entry and returns its value.
    pub fn remove(self) -> V {
        self.item.take().unwrap().1
    }
}

/// Entry which's key clashed.
#[derive(Debug)]
pub struct ClashEntry<'a, V> {
    key: NonZeroU64,
    item: &'a mut Item<V>,
}

impl<'a, V> ClashEntry<'a, V> {
    /// Returns the entry's currently used key.
    pub fn old_key(&self) -> NonZeroU64 {
        self.item.as_ref().unwrap().0
    }
    /// Returns the key that was used to find the entry.
    pub fn new_key(&self) -> NonZeroU64 {
        self.key
    }
    /// Keeps the old key and returns the entry.
    pub fn with_old_key(self) -> ExactEntry<'a, V> {
        ExactEntry { item: self.item }
    }
    /// Inserts the new key and returns the entry.
    pub fn with_new_key(self) -> ExactEntry<'a, V> {
        self.item.as_mut().unwrap().0 = self.key;
        ExactEntry { item: self.item }
    }
    /// Returns a reference to the entry's value.
    pub fn get(&self) -> &V {
        &self.item.as_ref().unwrap().1
    }
    /// Removes the entry and returns its value.
    pub fn remove(self) -> V {
        self.item.take().unwrap().1
    }
}

/// Entry which's key was not found.
#[derive(Debug)]
pub struct EmptyEntry<'a, V> {
    key: NonZeroU64,
    item: &'a mut Item<V>,
}

impl<'a, V> EmptyEntry<'a, V> {
    /// Returns the key that was used to find the entry.
    pub fn key(&self) -> NonZeroU64 {
        self.key
    }
    /// Inserts the value and returns the filled entry.
    pub fn insert(self, value: V) -> ExactEntry<'a, V> {
        let item = self.item;
        _ = item.insert((self.key, value));
        ExactEntry { item }
    }
}

/// Possible outcomes of looking up a key in the [`WeakHashMap`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyLookup<'a, V> {
    /// There is an item that exactly matches this key.
    Exact(NonZeroU64, &'a V),
    /// There is an item that conflicts with the key.
    Clash(NonZeroU64, &'a V),
    /// No items match the key.
    Empty,
}

impl<'a, V> KeyLookup<'a, V> {
    /// Returns a key value pair in case of an exact key match, otherwise returns `None`.
    pub fn exact(self) -> Option<(NonZeroU64, &'a V)> {
        if let Self::Exact(k, v) = self {
            Some((k, v))
        } else {
            None
        }
    }
    /// Returns a key value pair in case of a key clash, otherwise returns `None`.
    pub fn clash(self) -> Option<(NonZeroU64, &'a V)> {
        if let Self::Clash(k, v) = self {
            Some((k, v))
        } else {
            None
        }
    }
    /// Returns whether the key matched exactly.
    pub fn is_exact(&self) -> bool {
        matches!(self, Self::Exact(_, _))
    }
    /// Returns whether the key caused a conflict.
    pub fn is_clash(&self) -> bool {
        matches!(self, Self::Clash(_, _))
    }
    /// Returns whether the key was not present.
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }
}

impl<V> WeakHashMap<V> {
    /// Create a [`WeakHashMap`] that can hold a specified number of items.
    ///
    /// # Panics
    /// - Panics if `capacity` is zero.
    /// - Panics if `capacity * size_of::<V>()` exceeds `isize::MAX`.
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "must be able to hold at least one item!");
        let mut vec = Vec::new();
        vec.resize_with(capacity, || None);
        Self(vec.into_boxed_slice())
    }
    /// Returns the maximum number of items this [`WeakHashMap`] can hold at the same time.
    pub fn capacity(&self) -> usize {
        self.0.len()
    }
    /// Looks up an item and returns and entry for it.
    pub fn entry(&mut self, key: NonZeroU64) -> Entry<'_, V> {
        let item = &mut self.0[self.key_index(key)];
        match item {
            Some((k, _v)) => {
                if key == *k {
                    Entry::Exact(ExactEntry { item })
                } else {
                    Entry::Clash(ClashEntry { key, item })
                }
            }
            None => Entry::Empty(EmptyEntry { key, item }),
        }
    }
    /// Looks up an item and returns a reference to it.
    pub fn get(&self, key: NonZeroU64) -> KeyLookup<'_, V> {
        let item = &self.0[self.key_index(key)];
        match item {
            Some((k, v)) => {
                if key == *k {
                    KeyLookup::Exact(*k, v)
                } else {
                    KeyLookup::Clash(*k, v)
                }
            }
            None => KeyLookup::Empty,
        }
    }
    /// Remove all items form [`WeakHashMap`].
    pub fn clear(&mut self) {
        for opt in self.0.iter_mut() {
            _ = opt.take();
        }
    }
    // Returns the array index for the specified key.
    fn key_index(&self, key: NonZeroU64) -> usize {
        (key.get() % self.capacity() as u64) as usize
    }
}
