use std::fmt;
use std::num::NonZeroU64;
use std::ops::Rem;

/// Unique hash generated from a chess position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PositionHash(NonZeroU64);

impl PositionHash {
    pub fn new(hash: u64) -> Self {
        NonZeroU64::new(hash).map(Self).unwrap_or_default()
    }
}

impl Default for PositionHash {
    fn default() -> Self {
        Self(NonZeroU64::MAX)
    }
}

impl Rem<usize> for PositionHash {
    type Output = usize;

    fn rem(self, rhs: usize) -> Self::Output {
        const MAX: u64 = if size_of::<u64>() > size_of::<usize>() {
            usize::MAX as u64
        } else {
            u64::MAX
        };
        (self.0.get() & MAX) as usize % rhs
    }
}

impl fmt::Display for PositionHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
