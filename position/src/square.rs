use std::{fmt, str::FromStr};
use strum::{EnumCount, EnumIter, FromRepr, VariantArray};

/// File on a chess board.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumCount, EnumIter, VariantArray, FromRepr, Hash)]
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

impl FromStr for File {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "a" => File::A,
            "b" => File::B,
            "c" => File::C,
            "d" => File::D,
            "e" => File::E,
            "f" => File::F,
            "g" => File::G,
            "h" => File::H,
            _ => return Err(()),
        })
    }
}

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                File::A => "a",
                File::B => "b",
                File::C => "c",
                File::D => "d",
                File::E => "e",
                File::F => "f",
                File::G => "g",
                File::H => "h",
            }
        )
    }
}

/// Rank on a chess board.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumCount, EnumIter, VariantArray, FromRepr, Hash)]
pub enum Rank {
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
}

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Rank::R1 => "1",
                Rank::R2 => "2",
                Rank::R3 => "3",
                Rank::R4 => "4",
                Rank::R5 => "5",
                Rank::R6 => "6",
                Rank::R7 => "7",
                Rank::R8 => "8",
            }
        )
    }
}

impl FromStr for Rank {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "1" => Rank::R1,
            "2" => Rank::R2,
            "3" => Rank::R3,
            "4" => Rank::R4,
            "5" => Rank::R5,
            "6" => Rank::R6,
            "7" => Rank::R7,
            "8" => Rank::R8,
            _ => return Err(()),
        })
    }
}

/// Positive diagonal (bottom left to top right) on a chess board.
#[repr(i8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumCount, EnumIter, VariantArray, FromRepr, Hash)]
pub enum PosDiag {
    H1H1 = -(Rank::COUNT as i8) + 1,
    G1H2,
    F1H3,
    E1H4,
    D1H5,
    C1H6,
    B1H7,
    A1H8,
    A2G8,
    A3F8,
    A4E8,
    A5D8,
    A6C8,
    A7B8,
    A8A8,
}

impl fmt::Display for PosDiag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PosDiag::H1H1 => "h1-h1",
                PosDiag::G1H2 => "g1-h2",
                PosDiag::F1H3 => "f1-h3",
                PosDiag::E1H4 => "e1-h4",
                PosDiag::D1H5 => "d1-h5",
                PosDiag::C1H6 => "c1-h6",
                PosDiag::B1H7 => "b1-h7",
                PosDiag::A1H8 => "a1-h8",
                PosDiag::A2G8 => "a2-g8",
                PosDiag::A3F8 => "a3-f8",
                PosDiag::A4E8 => "a4-e8",
                PosDiag::A5D8 => "a5-d8",
                PosDiag::A6C8 => "a6-c8",
                PosDiag::A7B8 => "a7-b8",
                PosDiag::A8A8 => "a8-a8",
            }
        )
    }
}

/// Negative diagonal (top left to bottom right) on a chess board.
#[repr(i8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumCount, EnumIter, VariantArray, FromRepr, Hash)]
pub enum NegDiag {
    A1A1 = -(Rank::COUNT as i8) + 1,
    A2B1,
    A3C1,
    A4D1,
    A5E1,
    A6F1,
    A7G1,
    A8H1,
    B8H2,
    C8H3,
    D8H4,
    E8H5,
    F8H6,
    G8H7,
    H8H8,
}

impl fmt::Display for NegDiag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                NegDiag::A1A1 => "a1-a1",
                NegDiag::A2B1 => "a2-b1",
                NegDiag::A3C1 => "a3-c1",
                NegDiag::A4D1 => "a4-d1",
                NegDiag::A5E1 => "a5-e1",
                NegDiag::A6F1 => "a6-f1",
                NegDiag::A7G1 => "a7-g1",
                NegDiag::A8H1 => "a8-h1",
                NegDiag::B8H2 => "b8-h2",
                NegDiag::C8H3 => "c8-h3",
                NegDiag::D8H4 => "d8-h4",
                NegDiag::E8H5 => "e8-h5",
                NegDiag::F8H6 => "f8-h6",
                NegDiag::G8H7 => "g8-h7",
                NegDiag::H8H8 => "h8-h8",
            }
        )
    }
}

/// Square on a chess board.
///
/// # Examples
/// ```rust
/// use strum::{FromRepr, IntoEnumIterator};
/// use position::prelude::{Square, Rank, File};
///
/// // Conversion rule to rank/file
/// for square in Square::iter() {
///     assert_eq!(Some(square.rank()), Rank::from_repr(square as u8 / 8));
///     assert_eq!(Some(square.file()), File::from_repr(square as u8 % 8));
/// }
/// ```
///
/// ```rust
/// use strum::{FromRepr, IntoEnumIterator};
/// use position::prelude::{Square, PosDiag, NegDiag};
///
/// // Conversion rule to positive/negative diagonals
/// for square in Square::iter() {
///     let rank = square.rank();
///     let file = square.file();
///     assert_eq!(Some(square.pos_diag()), PosDiag::from_repr(rank as i8 - file as i8));
///     assert_eq!(Some(square.neg_diag()), NegDiag::from_repr(rank as i8 + file as i8 - 7));
/// }
/// ```
///
/// # See Also
/// [Rank]
/// [File]
/// [PosDiag]
/// [NegDiag]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumCount, EnumIter, VariantArray, FromRepr, Hash)]
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

    /// Cycles throug the [`Square`]'s enum several steps in a given direction.
    ///
    /// # Examples
    /// ```rust
    /// use position::prelude::Square;
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
}

impl FromStr for Square {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (file_str, rank_str) = s.split_at_checked(2).ok_or(())?;
        let file = file_str.parse::<File>()?;
        let rank = rank_str.parse::<Rank>()?;
        Ok(Square::new(rank, file))
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.file(), self.rank())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

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
