use strum::{EnumCount, FromRepr, VariantArray};

/// Index of a file on a chess board.
///
/// # Examples
/// ```rust
/// use strum::FromRepr;
/// use position::prelude::File;
///
/// assert_eq!(File::from_repr(0), Some(File::A));
/// assert_eq!(File::from_repr(1), Some(File::B));
/// assert_eq!(File::from_repr(2), Some(File::C));
/// assert_eq!(File::from_repr(3), Some(File::D));
/// assert_eq!(File::from_repr(4), Some(File::E));
/// assert_eq!(File::from_repr(5), Some(File::F));
/// assert_eq!(File::from_repr(6), Some(File::G));
/// assert_eq!(File::from_repr(7), Some(File::H));
/// ```
///
/// # See Also
/// [Square]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumCount, VariantArray, FromRepr)]
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

///  Index of a rank on a chess board.
///
/// # Examples
/// ```rust
/// use strum::FromRepr;
/// use position::prelude::Rank;
///
/// assert_eq!(Rank::from_repr(0), Some(Rank::R1));
/// assert_eq!(Rank::from_repr(1), Some(Rank::R2));
/// assert_eq!(Rank::from_repr(2), Some(Rank::R3));
/// assert_eq!(Rank::from_repr(3), Some(Rank::R4));
/// assert_eq!(Rank::from_repr(4), Some(Rank::R5));
/// assert_eq!(Rank::from_repr(5), Some(Rank::R6));
/// assert_eq!(Rank::from_repr(6), Some(Rank::R7));
/// assert_eq!(Rank::from_repr(7), Some(Rank::R8));
/// ```
///
/// # See Also
/// [Square]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumCount, VariantArray, FromRepr)]
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

/// Index of a positive (bottom left to top right) diagonal on a chess board.
///
/// # Examples
/// ```rust
/// use strum::FromRepr;
/// use position::prelude::PosDiag;
///
/// assert_eq!(PosDiag::from_repr(-7), Some(PosDiag::H1H1));
/// assert_eq!(PosDiag::from_repr(-6), Some(PosDiag::G1H2));
/// assert_eq!(PosDiag::from_repr(-5), Some(PosDiag::F1H3));
/// assert_eq!(PosDiag::from_repr(-4), Some(PosDiag::E1H4));
/// assert_eq!(PosDiag::from_repr(-3), Some(PosDiag::D1H5));
/// assert_eq!(PosDiag::from_repr(-2), Some(PosDiag::C1H6));
/// assert_eq!(PosDiag::from_repr(-1), Some(PosDiag::B1H7));
/// assert_eq!(PosDiag::from_repr(0), Some(PosDiag::A1H8));
/// assert_eq!(PosDiag::from_repr(1), Some(PosDiag::A2G8));
/// assert_eq!(PosDiag::from_repr(2), Some(PosDiag::A3F8));
/// assert_eq!(PosDiag::from_repr(3), Some(PosDiag::A4E8));
/// assert_eq!(PosDiag::from_repr(4), Some(PosDiag::A5D8));
/// assert_eq!(PosDiag::from_repr(5), Some(PosDiag::A6C8));
/// assert_eq!(PosDiag::from_repr(6), Some(PosDiag::A7B8));
/// assert_eq!(PosDiag::from_repr(7), Some(PosDiag::A8A8));
/// ```
///
/// # See Also
/// [Square]
#[repr(i8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumCount, VariantArray, FromRepr)]
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

/// Index of a negative (top left to bottom right) diagonal on a chess board.
///
/// # Examples
/// ```rust
/// use strum::FromRepr;
/// use position::prelude::NegDiag;
///
/// assert_eq!(NegDiag::from_repr(-7), Some(NegDiag::A1A1));
/// assert_eq!(NegDiag::from_repr(-6), Some(NegDiag::A2B1));
/// assert_eq!(NegDiag::from_repr(-5), Some(NegDiag::A3C1));
/// assert_eq!(NegDiag::from_repr(-4), Some(NegDiag::A4D1));
/// assert_eq!(NegDiag::from_repr(-3), Some(NegDiag::A5E1));
/// assert_eq!(NegDiag::from_repr(-2), Some(NegDiag::A6F1));
/// assert_eq!(NegDiag::from_repr(-1), Some(NegDiag::A7G1));
/// assert_eq!(NegDiag::from_repr(0), Some(NegDiag::A8H1));
/// assert_eq!(NegDiag::from_repr(1), Some(NegDiag::B8H2));
/// assert_eq!(NegDiag::from_repr(2), Some(NegDiag::C8H3));
/// assert_eq!(NegDiag::from_repr(3), Some(NegDiag::D8H4));
/// assert_eq!(NegDiag::from_repr(4), Some(NegDiag::E8H5));
/// assert_eq!(NegDiag::from_repr(5), Some(NegDiag::F8H6));
/// assert_eq!(NegDiag::from_repr(6), Some(NegDiag::G8H7));
/// assert_eq!(NegDiag::from_repr(7), Some(NegDiag::H8H8));
/// ```
///
/// # See Also
/// [Square]
#[repr(i8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumCount, VariantArray, FromRepr)]
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

/// Index of a square on a chess board.
///
/// # Examples
/// ```rust
/// use strum::{FromRepr, VariantArray};
/// use position::prelude::{Square, Rank, File};
///
/// // Conversion rule to rank/file
/// for &square in Square::VARIANTS {
///     assert_eq!(Some(square.rank()), Rank::from_repr(square as u8 / 8));
///     assert_eq!(Some(square.file()), File::from_repr(square as u8 % 8));
/// }
/// ```
///
/// ```rust
/// use strum::{FromRepr, VariantArray};
/// use position::prelude::{Square, PosDiag, NegDiag};
///
/// // Conversion rule to positive/negative diagonals
/// for &square in Square::VARIANTS {
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumCount, VariantArray, FromRepr)]
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
    /// Creates a square based on it's rank and file.
    #[inline(always)]
    #[must_use]
    pub const fn new(rank: Rank, file: File) -> Self {
        Self::from_repr(rank as u8 * File::COUNT as u8 + file as u8).unwrap()
    }

    /// Returns file of the square.
    ///
    /// # Returns
    /// `File` - file of the square
    #[inline(always)]
    #[must_use]
    pub const fn file(self) -> File {
        File::from_repr(self as u8 % File::COUNT as u8).unwrap()
    }

    /// Returns rank of the square.
    ///
    /// # Returns
    /// `Rank` - rank of the square
    #[inline(always)]
    #[must_use]
    pub const fn rank(self) -> Rank {
        Rank::from_repr(self as u8 / File::COUNT as u8).unwrap()
    }

    /// Returns positive diagonal of the square.
    ///
    /// # Returns
    /// `PosDiag` - positive diagonal of the square
    #[inline(always)]
    #[must_use]
    pub const fn pos_diag(self) -> PosDiag {
        PosDiag::from_repr(self.rank() as i8 - self.file() as i8).unwrap()
    }

    /// Returns negative diagonal of the square.
    ///
    /// # Returns
    /// `NegDiag` - negative diagonal of the square
    #[inline(always)]
    #[must_use]
    pub const fn neg_diag(self) -> NegDiag {
        NegDiag::from_repr(self.rank() as i8 + self.file() as i8 + NegDiag::A1A1 as i8).unwrap()
    }

    /// Shifts square's index by delta.
    ///
    /// # Arguments
    /// * `delta` - the signed amount by which to shift the index
    ///
    /// # Returns
    /// `Self` - resulting square
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_square_constructor() {
        for &rank in Rank::VARIANTS {
            for &file in File::VARIANTS {
                let square = Square::new(rank, file);
                assert_eq!(square.rank(), rank);
                assert_eq!(square.file(), file);
            }
        }
    }
}
