use grob_core::game::walker::GameEnding;

use crate::MAX_DEPTH;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Score(pub i32);

impl Score {
    /// The maximum representable score in centipawns.
    pub const CENTIPAWNS_MAX: i32 = i32::MAX - MAX_DEPTH as i32 - 1;
    /// The minimum representable score in centipawns.
    pub const CENTIPAWNS_MIN: i32 = i32::MIN + MAX_DEPTH as i32 + 1;

    /// Creates a new score for mating in n turns.
    pub fn from_mating(n_turns: u8) -> Self {
        Self(i32::MAX - n_turns as i32)
    }
    /// Creates a new score for being mated in n turns.
    pub fn from_mated(n_turns: u8) -> Self {
        Self(i32::MIN + n_turns as i32)
    }
    /// Creates a new score in centipawns.
    ///
    /// The score will be clamped to \[[`Self::CENTIPAWNS_MIN`], [`Self::CENTIPAWNS_MAX`]\].
    pub fn from_centipawns(value: i32) -> Self {
        Self(value.clamp(Self::CENTIPAWNS_MIN, Self::CENTIPAWNS_MAX))
    }
    /// Returns `true` if the score represents the position where the player is mating.
    pub fn is_mating(self) -> bool {
        self.0 > Self::CENTIPAWNS_MAX
    }
    /// Returns `true` if the score represents the position where the player is being mated.
    pub fn is_mated(self) -> bool {
        self.0 < Self::CENTIPAWNS_MIN
    }
    /// Returns `true` if the score is measured in centipawns (not a "mating" or "mated" score).
    pub fn is_centipawns(self) -> bool {
        !self.is_mating() && !self.is_mated()
    }
    /// Returns the number of turns for the "mating" position.
    pub fn as_mating(self) -> Option<u8> {
        self.is_mating().then_some((i32::MAX - self.0) as u8)
    }
    /// Returns the number of turns for the "being mated" position.
    pub fn as_mated(self) -> Option<u8> {
        self.is_mating().then_some((i32::MAX - self.0) as u8)
    }
    /// Returns the value of the position in centipawns
    /// unless it's a "mating" or "being mated" position.
    pub fn as_centipawns(self) -> Option<i32> {
        self.is_centipawns().then_some(self.0)
    }
    /// Returns the socre for the [`GameEnding`].
    pub fn from_ending(ending: GameEnding) -> Self {
        match ending {
            GameEnding::Stalemate => Self::from_centipawns(0),
            GameEnding::Checkmate => Self::from_mated(0),
        }
    }
    /// Returns the score for the other player on the previous turn.
    pub fn prev(self) -> Self {
        if self.is_mating() {
            Self(-self.0 - 1)
        } else {
            Self(-self.0)
        }
    }
    /// Returns the score for the other player on the next turn.
    pub fn next(self) -> Self {
        if self.is_mating() {
            Self(-self.0 - 2)
        } else if self.is_mated() {
            Self(-(self.0 + 1))
        } else {
            Self(-self.0)
        }
    }
}
