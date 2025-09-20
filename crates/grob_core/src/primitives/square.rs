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
pub enum File {
    /// The 'A' file.
    #[strum(ascii_case_insensitive)]
    A,
    /// The 'B' file.
    #[strum(ascii_case_insensitive)]
    B,
    /// The 'C' file.
    #[strum(ascii_case_insensitive)]
    C,
    /// The 'D' file.
    #[strum(ascii_case_insensitive)]
    D,
    /// The 'E' file.
    #[strum(ascii_case_insensitive)]
    E,
    /// The 'F' file.
    #[strum(ascii_case_insensitive)]
    F,
    /// The 'G' file.
    #[strum(ascii_case_insensitive)]
    G,
    /// The 'H' file.
    #[strum(ascii_case_insensitive)]
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
pub enum PosDiag {
    #[strum(ascii_case_insensitive)]
    H1H1 = -(Rank::COUNT as i8) + 1,
    #[strum(ascii_case_insensitive)]
    G1H2,
    #[strum(ascii_case_insensitive)]
    F1H3,
    #[strum(ascii_case_insensitive)]
    E1H4,
    #[strum(ascii_case_insensitive)]
    D1H5,
    #[strum(ascii_case_insensitive)]
    C1H6,
    #[strum(ascii_case_insensitive)]
    B1H7,
    /// The main diagonal.
    #[strum(ascii_case_insensitive)]
    A1H8,
    #[strum(ascii_case_insensitive)]
    A2G8,
    #[strum(ascii_case_insensitive)]
    A3F8,
    #[strum(ascii_case_insensitive)]
    A4E8,
    #[strum(ascii_case_insensitive)]
    A5D8,
    #[strum(ascii_case_insensitive)]
    A6C8,
    #[strum(ascii_case_insensitive)]
    A7B8,
    #[strum(ascii_case_insensitive)]
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
pub enum NegDiag {
    #[strum(ascii_case_insensitive)]
    A1A1 = -(Rank::COUNT as i8) + 1,
    #[strum(ascii_case_insensitive)]
    A2B1,
    #[strum(ascii_case_insensitive)]
    A3C1,
    #[strum(ascii_case_insensitive)]
    A4D1,
    #[strum(ascii_case_insensitive)]
    A5E1,
    #[strum(ascii_case_insensitive)]
    A6F1,
    #[strum(ascii_case_insensitive)]
    A7G1,
    /// The main antidiagonal.
    #[strum(ascii_case_insensitive)]
    A8H1,
    #[strum(ascii_case_insensitive)]
    B8H2,
    #[strum(ascii_case_insensitive)]
    C8H3,
    #[strum(ascii_case_insensitive)]
    D8H4,
    #[strum(ascii_case_insensitive)]
    E8H5,
    #[strum(ascii_case_insensitive)]
    F8H6,
    #[strum(ascii_case_insensitive)]
    G8H7,
    #[strum(ascii_case_insensitive)]
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
pub enum Square {
    #[strum(ascii_case_insensitive)]
    A1,
    #[strum(ascii_case_insensitive)]
    B1,
    #[strum(ascii_case_insensitive)]
    C1,
    #[strum(ascii_case_insensitive)]
    D1,
    #[strum(ascii_case_insensitive)]
    E1,
    #[strum(ascii_case_insensitive)]
    F1,
    #[strum(ascii_case_insensitive)]
    G1,
    #[strum(ascii_case_insensitive)]
    H1,
    #[strum(ascii_case_insensitive)]
    A2,
    #[strum(ascii_case_insensitive)]
    B2,
    #[strum(ascii_case_insensitive)]
    C2,
    #[strum(ascii_case_insensitive)]
    D2,
    #[strum(ascii_case_insensitive)]
    E2,
    #[strum(ascii_case_insensitive)]
    F2,
    #[strum(ascii_case_insensitive)]
    G2,
    #[strum(ascii_case_insensitive)]
    H2,
    #[strum(ascii_case_insensitive)]
    A3,
    #[strum(ascii_case_insensitive)]
    B3,
    #[strum(ascii_case_insensitive)]
    C3,
    #[strum(ascii_case_insensitive)]
    D3,
    #[strum(ascii_case_insensitive)]
    E3,
    #[strum(ascii_case_insensitive)]
    F3,
    #[strum(ascii_case_insensitive)]
    G3,
    #[strum(ascii_case_insensitive)]
    H3,
    #[strum(ascii_case_insensitive)]
    A4,
    #[strum(ascii_case_insensitive)]
    B4,
    #[strum(ascii_case_insensitive)]
    C4,
    #[strum(ascii_case_insensitive)]
    D4,
    #[strum(ascii_case_insensitive)]
    E4,
    #[strum(ascii_case_insensitive)]
    F4,
    #[strum(ascii_case_insensitive)]
    G4,
    #[strum(ascii_case_insensitive)]
    H4,
    #[strum(ascii_case_insensitive)]
    A5,
    #[strum(ascii_case_insensitive)]
    B5,
    #[strum(ascii_case_insensitive)]
    C5,
    #[strum(ascii_case_insensitive)]
    D5,
    #[strum(ascii_case_insensitive)]
    E5,
    #[strum(ascii_case_insensitive)]
    F5,
    #[strum(ascii_case_insensitive)]
    G5,
    #[strum(ascii_case_insensitive)]
    H5,
    #[strum(ascii_case_insensitive)]
    A6,
    #[strum(ascii_case_insensitive)]
    B6,
    #[strum(ascii_case_insensitive)]
    C6,
    #[strum(ascii_case_insensitive)]
    D6,
    #[strum(ascii_case_insensitive)]
    E6,
    #[strum(ascii_case_insensitive)]
    F6,
    #[strum(ascii_case_insensitive)]
    G6,
    #[strum(ascii_case_insensitive)]
    H6,
    #[strum(ascii_case_insensitive)]
    A7,
    #[strum(ascii_case_insensitive)]
    B7,
    #[strum(ascii_case_insensitive)]
    C7,
    #[strum(ascii_case_insensitive)]
    D7,
    #[strum(ascii_case_insensitive)]
    E7,
    #[strum(ascii_case_insensitive)]
    F7,
    #[strum(ascii_case_insensitive)]
    G7,
    #[strum(ascii_case_insensitive)]
    H7,
    #[strum(ascii_case_insensitive)]
    A8,
    #[strum(ascii_case_insensitive)]
    B8,
    #[strum(ascii_case_insensitive)]
    C8,
    #[strum(ascii_case_insensitive)]
    D8,
    #[strum(ascii_case_insensitive)]
    E8,
    #[strum(ascii_case_insensitive)]
    F8,
    #[strum(ascii_case_insensitive)]
    G8,
    #[strum(ascii_case_insensitive)]
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
    /// use grob_core::Square;
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
