use crate::GameEnding;

/// How advantageous is a chess position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Score {
    /// Length of a forced mate sequence in turns (good).
    Mating(u64),
    /// Length of a forced mate sequence in turns (bad).
    Mated(u64),
    /// Score in centi-pawns (1/100 of a pawn).
    Cp(i32),
}

impl Score {
    /// Returns the socre for the [`GameEnding`].
    pub fn ending(ending: GameEnding) -> Self {
        match ending {
            GameEnding::Stalemate => Self::Cp(0),
            GameEnding::Checkmate => Self::Mated(0),
        }
    }
    /// Returns the score for the other player on the previous turn.
    pub fn prev(self) -> Self {
        match self {
            Score::Mating(n) => Score::Mated(n),
            Score::Mated(n) => Score::Mating(n + 1),
            Score::Cp(i) => Score::Cp(-i),
        }
    }
    /// Returns the score for the other player on the next turn.
    pub fn next(self) -> Self {
        match self {
            Score::Mating(n) => Score::Mated(n - 1),
            Score::Mated(n) => Score::Mating(n),
            Score::Cp(i) => Score::Cp(-i),
        }
    }
}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Score {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Score::Mating(n1), Score::Mating(n2)) => n1.cmp(n2).reverse(),
            (Score::Mating(_), Score::Mated(_)) => std::cmp::Ordering::Greater,
            (Score::Mating(_), Score::Cp(_)) => std::cmp::Ordering::Greater,
            (Score::Mated(_), Score::Mating(_)) => std::cmp::Ordering::Less,
            (Score::Mated(n1), Score::Mated(n2)) => n1.cmp(n2),
            (Score::Mated(_), Score::Cp(_)) => std::cmp::Ordering::Less,
            (Score::Cp(_), Score::Mating(_)) => std::cmp::Ordering::Less,
            (Score::Cp(_), Score::Mated(_)) => std::cmp::Ordering::Greater,
            (Score::Cp(i1), Score::Cp(i2)) => i1.cmp(i2),
        }
    }
}
