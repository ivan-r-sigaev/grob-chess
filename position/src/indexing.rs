use std::{fmt, ops::Not, str::FromStr};
use strum::{EnumCount, EnumIter, FromRepr, VariantArray};

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

/// Index of a square on a chess board.
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

/// Color of a chess piece.
///
/// # Examples
/// ```rust
/// use position::prelude::Color;
/// use strum::FromRepr;
///
/// assert_eq!(Color::from_repr(0), Some(Color::White));
/// assert_eq!(Color::from_repr(1), Some(Color::Black));
/// ```
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumCount, EnumIter, VariantArray, FromRepr, Hash)]
pub enum Color {
    White,
    Black,
}

impl Color {
    /// Returns the rank where the pawns of this color can promote.
    ///
    /// # Returns
    /// `Rank` - the rank where the pawns of this color can promote
    ///
    /// # Examples
    /// ```rust
    /// use position::prelude::{Color, Rank};
    ///
    /// assert_eq!(Color::White.promotion_rank(), Rank::R8);
    /// assert_eq!(Color::Black.promotion_rank(), Rank::R1);
    /// ```
    #[must_use]
    pub fn promotion_rank(self) -> Rank {
        if self == Color::White {
            Rank::R8
        } else {
            Rank::R1
        }
    }

    /// Returns the rank where the pawns of this color will be at the start of the game.
    ///
    /// # Returns
    /// `Rank` - the rank where the pawns of this color will be at the start of the game.
    ///
    /// # Examples
    /// ```rust
    /// use position::prelude::{Color, Rank};
    ///
    /// assert_eq!(Color::White.pawn_rank(), Rank::R2);
    /// assert_eq!(Color::Black.pawn_rank(), Rank::R7);
    /// ```
    #[must_use]
    pub fn pawn_rank(self) -> Rank {
        if self == Color::White {
            Rank::R2
        } else {
            Rank::R7
        }
    }

    /// Returns the rank where the pawns of this color will be after completing en passant.
    ///
    /// # Returns
    /// `Rank` - the rank where the pawns of this color will be after completing en passant.
    ///
    /// # Examples
    /// ```rust
    /// use position::prelude::{Color, Rank};
    ///
    /// assert_eq!(Color::White.en_passant_dest_rank(), Rank::R6);
    /// assert_eq!(Color::Black.en_passant_dest_rank(), Rank::R3);
    /// ```
    #[must_use]
    pub fn en_passant_dest_rank(self) -> Rank {
        if self == Color::White {
            Rank::R6
        } else {
            Rank::R3
        }
    }
}

impl Not for Color {
    type Output = Self;

    /// Returns the opposite color.
    ///
    /// # Returns
    /// `Self` - the opposite color
    ///
    /// # Examples
    /// ```rust
    /// use position::prelude::Color;
    ///
    /// assert_eq!(!Color::White, Color::Black);
    /// assert_eq!(!Color::Black, Color::White);
    /// ```
    #[inline(always)]
    fn not(self) -> Self::Output {
        Self::from_repr((self as u8 + 1) % 2).unwrap()
    }
}

impl FromStr for Color {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "w" => Color::White,
            "b" => Color::Black,
            _ => return Err(()),
        })
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Color::White => "w",
                Color::Black => "b",
            }
        )
    }
}

/// A type of a chess piece.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumCount, EnumIter, VariantArray, FromRepr, Hash)]
pub enum Piece {
    Pawn,
    Bishop,
    Knight,
    Rook,
    Queen,
    King,
}

impl Piece {
    /// Get promotion to given piece if possible.
    ///
    /// # Returns
    ///
    /// `Option<Promotion>`:
    /// - `Some(promotion: Promotion)` - promotion to piece
    /// - `None` - it's not possible to promote to this piece
    pub fn promotion(self) -> Option<Promotion> {
        Some(match self {
            Piece::Bishop => Promotion::Bishop,
            Piece::Knight => Promotion::Knight,
            Piece::Rook => Promotion::Rook,
            Piece::Queen => Promotion::Queen,
            _ => return None,
        })
    }
}

impl FromStr for Piece {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "p" => Piece::Pawn,
            "b" => Piece::Bishop,
            "n" => Piece::Knight,
            "r" => Piece::Rook,
            "q" => Piece::Queen,
            "k" => Piece::King,
            _ => return Err(()),
        })
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Piece::Pawn => "p",
                Piece::Bishop => "b",
                Piece::Knight => "n",
                Piece::Rook => "r",
                Piece::Queen => "q",
                Piece::King => "k",
            }
        )
    }
}

/// A type of possible pawn promotion.
#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumCount, EnumIter, VariantArray, FromRepr, Hash)]
pub enum Promotion {
    Bishop,
    Knight,
    Rook,
    Queen,
}

impl Promotion {
    /// Converts promotion to unerlying piece.
    ///
    /// # Returns
    ///
    /// `Piece` - the piece to promote to
    pub fn piece(self) -> Piece {
        match self {
            Promotion::Bishop => Piece::Bishop,
            Promotion::Knight => Piece::Knight,
            Promotion::Rook => Piece::Rook,
            Promotion::Queen => Piece::Queen,
        }
    }
}

impl FromStr for Promotion {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let piece = s.parse::<Piece>()?;
        piece.promotion().ok_or(())
    }
}

impl fmt::Display for Promotion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.piece())
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
