use std::{num::NonZeroU64, ops::Deref};

/// The underlying type for the transposition table.
#[derive(Debug, Clone)]
pub struct HashTable<T>(Box<[Option<Item<T>>]>);

type Item<T> = (NonZeroU64, T);

/// An immutable reference into the hashmap.
#[derive(Debug, Clone)]
pub struct Ref<'a, T> {
    key: NonZeroU64,
    item: &'a Item<T>,
}

impl<T> Ref<'_, T> {
    /// Returns the currently used key of the item.
    pub fn key(&self) -> NonZeroU64 {
        self.item.0
    }
    /// Returns the key that was used to find the item.
    pub fn matched_key(&self) -> NonZeroU64 {
        self.key
    }
    /// Returns `true` if the key and the search key are the same.
    pub fn is_exact(&self) -> bool {
        self.key() == self.matched_key()
    }
    /// Returns the reference to the item's value.
    pub fn get(&self) -> &T {
        &self.item.1
    }
}

impl<T> Deref for Ref<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<T> HashTable<T> {
    /// Size of a single item slot in bytes.
    pub const SLOT_SIZE: usize = size_of::<Option<Item<T>>>();

    /// Constructs a new hashmap that can hold no more than `capacity` number of items.
    ///
    /// # Panics
    /// Panics if `capacity` is zero.
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "must be able to hold at least one item!");
        let mut vec = Vec::new();
        vec.resize_with(capacity, || None);
        Self(vec.into_boxed_slice())
    }
    /// Returns the maximum nubmer of items this hashmap can hold.
    pub fn capacity(&self) -> usize {
        self.0.len()
    }
    /// Returns the reference the item of the hashmap.
    pub fn get(&self, key: NonZeroU64) -> Option<Ref<'_, T>> {
        self.item(key).as_ref().map(|item| Ref { key, item })
    }
    /// Inserts the item into the hashmap.
    ///
    /// This will overwrite the item with the matching key.
    pub fn insert(&mut self, key: NonZeroU64, value: T) -> Option<T> {
        self.item_mut(key).replace((key, value)).map(|(_, v)| v)
    }
    /// Removes all items from the hashmap.
    pub fn clear(&mut self) {
        self.0.fill_with(|| None);
    }
    /// Returns a raw reference to the hashmap's item.
    fn item(&self, key: NonZeroU64) -> &Option<Item<T>> {
        &self.0[self.key_index(key)]
    }
    /// Returns a raw mutable reference to the hashmap's item.
    fn item_mut(&mut self, key: NonZeroU64) -> &mut Option<Item<T>> {
        &mut self.0[self.key_index(key)]
    }
    /// Returns the internal array index for the specified key.
    fn key_index(&self, key: NonZeroU64) -> usize {
        (key.get() % self.capacity() as u64) as usize
    }
}
