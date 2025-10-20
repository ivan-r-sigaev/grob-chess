use strum::{Display, EnumCount, EnumIter, EnumString, FromRepr, VariantArray};

/// File on a chess board.
#[repr(u8)]
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    EnumCount,
    EnumIter,
    Display,
    EnumString,
    VariantArray,
    FromRepr,
    Hash,
)]
#[strum(ascii_case_insensitive)]
pub enum File {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
}

/// Rank on a chess board.
#[repr(u8)]
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    EnumCount,
    EnumIter,
    Display,
    EnumString,
    VariantArray,
    FromRepr,
    Hash,
)]
pub enum Rank {
    /// The first rank.
    #[strum(serialize = "1")]
    R1,
    /// The second rank.
    #[strum(serialize = "2")]
    R2,
    /// The third rank.
    #[strum(serialize = "3")]
    R3,
    /// The fourth rank.
    #[strum(serialize = "4")]
    R4,
    /// The fifth rank.
    #[strum(serialize = "5")]
    R5,
    /// The sixth rank.
    #[strum(serialize = "6")]
    R6,
    /// The seventh rank.
    #[strum(serialize = "7")]
    R7,
    /// The eighth rank.
    #[strum(serialize = "8")]
    R8,
}

impl Rank {
    /// Mirrors rank horizontally.
    pub const fn mirrored(self) -> Self {
        Self::from_repr((Self::COUNT - 1) as u8 - self as u8).unwrap()
    }
}

/// Positive diagonal (bottom left to top right) on a chess board.
#[repr(i8)]
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    EnumCount,
    EnumIter,
    Display,
    EnumString,
    VariantArray,
    FromRepr,
    Hash,
)]
#[strum(ascii_case_insensitive)]
pub enum PosDiag {
    H1H1 = -(Rank::COUNT as i8) + 1,
    G1H2,
    F1H3,
    E1H4,
    D1H5,
    C1H6,
    B1H7,
    /// The main diagonal.
    A1H8,
    A2G8,
    A3F8,
    A4E8,
    A5D8,
    A6C8,
    A7B8,
    A8A8,
}

/// Negative diagonal (top left to bottom right) on a chess board.
#[repr(i8)]
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    EnumCount,
    EnumIter,
    Display,
    EnumString,
    VariantArray,
    FromRepr,
    Hash,
)]
#[strum(ascii_case_insensitive)]
pub enum NegDiag {
    A1A1 = -(Rank::COUNT as i8) + 1,
    A2B1,
    A3C1,
    A4D1,
    A5E1,
    A6F1,
    A7G1,
    /// The main antidiagonal.
    A8H1,
    B8H2,
    C8H3,
    D8H4,
    E8H5,
    F8H6,
    G8H7,
    H8H8,
}

/// Square on a chess board.
#[repr(u8)]
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    EnumCount,
    EnumIter,
    Display,
    EnumString,
    VariantArray,
    FromRepr,
    Hash,
)]
#[strum(ascii_case_insensitive)]
pub enum Square {
    A1,
    B1,
    C1,
    D1,
    E1,
    F1,
    G1,
    H1,
    A2,
    B2,
    C2,
    D2,
    E2,
    F2,
    G2,
    H2,
    A3,
    B3,
    C3,
    D3,
    E3,
    F3,
    G3,
    H3,
    A4,
    B4,
    C4,
    D4,
    E4,
    F4,
    G4,
    H4,
    A5,
    B5,
    C5,
    D5,
    E5,
    F5,
    G5,
    H5,
    A6,
    B6,
    C6,
    D6,
    E6,
    F6,
    G6,
    H6,
    A7,
    B7,
    C7,
    D7,
    E7,
    F7,
    G7,
    H7,
    A8,
    B8,
    C8,
    D8,
    E8,
    F8,
    G8,
    H8,
}

impl Square {
    /// Constructs new square from rank and file.
    #[inline(always)]
    #[must_use]
    pub const fn new(rank: Rank, file: File) -> Self {
        Self::from_repr(rank as u8 * File::COUNT as u8 + file as u8).unwrap()
    }

    /// Returns file of the square
    #[inline(always)]
    #[must_use]
    pub const fn file(self) -> File {
        File::from_repr(self as u8 % File::COUNT as u8).unwrap()
    }

    /// Returns rank of the square.
    #[inline(always)]
    #[must_use]
    pub const fn rank(self) -> Rank {
        Rank::from_repr(self as u8 / File::COUNT as u8).unwrap()
    }

    /// Returns positive diagonal of the square.
    #[inline(always)]
    #[must_use]
    pub const fn pos_diag(self) -> PosDiag {
        PosDiag::from_repr(self.rank() as i8 - self.file() as i8).unwrap()
    }

    /// Returns negative diagonal of the square.
    #[inline(always)]
    #[must_use]
    pub const fn neg_diag(self) -> NegDiag {
        NegDiag::from_repr(self.rank() as i8 + self.file() as i8 + NegDiag::A1A1 as i8).unwrap()
    }

    /// Cycles through the [`Square`]'s enum several steps in a given direction.
    ///
    /// # Examples
    /// ```rust
    /// use grob_core::square::Square;
    ///
    /// assert_eq!(Square::A1.shifted(1), Square::B1);
    /// assert_eq!(Square::A1.shifted(-1), Square::H8);
    ///
    /// assert_eq!(Square::B1.shifted(-1), Square::A1);
    /// assert_eq!(Square::H8.shifted(1), Square::A1);
    ///
    /// assert_eq!(Square::A1.shifted(8), Square::A2);
    /// assert_eq!(Square::A1.shifted(-8), Square::A8);
    ///
    /// assert_eq!(Square::A1.shifted(64), Square::A1);
    /// assert_eq!(Square::A1.shifted(-64), Square::A1);
    ///
    /// assert_eq!(Square::A1.shifted(64 + 1), Square::B1);
    /// assert_eq!(Square::A1.shifted(-64 - 1), Square::H8);
    /// ```
    #[inline(always)]
    #[must_use]
    pub const fn shifted(self, delta: i8) -> Self {
        Self::from_repr(
            (self as i8)
                .wrapping_add(delta)
                .rem_euclid(Self::COUNT as i8) as u8,
        )
        .unwrap()
    }

    /// Mirrors square horizontally.
    pub const fn mirrored(self) -> Self {
        Self::new(self.rank().mirrored(), self.file())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    #[test]
    fn test_rank_file_conversion() {
        for square in Square::iter() {
            assert_eq!(Some(square.rank()), Rank::from_repr(square as u8 / 8));
            assert_eq!(Some(square.file()), File::from_repr(square as u8 % 8));
        }
    }

    #[test]
    fn test_pos_neg_diagonals() {
        for square in Square::iter() {
            let rank = square.rank();
            let file = square.file();
            assert_eq!(
                Some(square.pos_diag()),
                PosDiag::from_repr(rank as i8 - file as i8)
            );
            assert_eq!(
                Some(square.neg_diag()),
                NegDiag::from_repr(rank as i8 + file as i8 - 7)
            );
        }
    }

    #[test]
    fn test_square_constructor() {
        for rank in Rank::iter() {
            for file in File::iter() {
                let square = Square::new(rank, file);
                assert_eq!(square.rank(), rank);
                assert_eq!(square.file(), file);
            }
        }
    }
}
