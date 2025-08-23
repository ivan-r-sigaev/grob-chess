use crate::{Rank, Square};
use std::ops::Not;
use strum::{Display, EnumCount, EnumIter, EnumString, FromRepr, VariantArray};

/// Color of a chess piece.
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
pub enum Color {
    /// White pieces.
    #[strum(serialize = "w")]
    #[strum(ascii_case_insensitive)]
    White,
    /// Black pieces.
    #[strum(serialize = "b")]
    #[strum(ascii_case_insensitive)]
    Black,
}

impl Color {
    /// Returns the same square for white and mirrors the square for black.
    pub fn mirror_square(self, square: Square) -> Square {
        match self {
            Color::White => square,
            Color::Black => square.mirrored(),
        }
    }

    /// Returns the same rank for white and mirrors the rank for black.
    pub fn mirror_rank(self, rank: Rank) -> Rank {
        match self {
            Color::White => rank,
            Color::Black => rank.mirrored(),
        }
    }
}

impl Not for Color {
    type Output = Self;

    /// Returns the opposite color.
    ///
    /// # Examples
    /// ```rust
    /// use board::Color;
    ///
    /// assert_eq!(!Color::White, Color::Black);
    /// assert_eq!(!Color::Black, Color::White);
    /// ```
    #[inline(always)]
    fn not(self) -> Self::Output {
        Self::from_repr((self as u8 + 1) % 2).unwrap()
    }
}

/// Chess piece type.
#[repr(u8)]
#[derive(
    Clone,
    Copy,
    Debug,
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
pub enum Piece {
    /// Pawn pieces.
    #[strum(serialize = "p")]
    #[strum(ascii_case_insensitive)]
    Pawn,
    /// Bishop pieces.
    #[strum(serialize = "b")]
    #[strum(ascii_case_insensitive)]
    Bishop,
    /// Knight pieces.
    #[strum(serialize = "n")]
    #[strum(ascii_case_insensitive)]
    Knight,
    /// Rook pieces.
    #[strum(serialize = "r")]
    #[strum(ascii_case_insensitive)]
    Rook,
    /// Queen pieces.
    #[strum(serialize = "q")]
    #[strum(ascii_case_insensitive)]
    Queen,
    /// King pieces.
    #[strum(serialize = "k")]
    #[strum(ascii_case_insensitive)]
    King,
}

impl Piece {
    /// Convert a [`Piece`] to [`Promotion`] if pawns can promote to this kind of piece.
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

/// Chess piece type that pawns can promote to.
#[derive(
    Clone,
    Copy,
    Debug,
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
pub enum Promotion {
    /// Promotion to bishop.
    #[strum(serialize = "b")]
    #[strum(ascii_case_insensitive)]
    Bishop,
    /// Promotion to knight.
    #[strum(serialize = "n")]
    #[strum(ascii_case_insensitive)]
    Knight,
    /// Promotion to rook.
    #[strum(serialize = "r")]
    #[strum(ascii_case_insensitive)]
    Rook,
    /// Promotion to queen.
    #[strum(serialize = "q")]
    #[strum(ascii_case_insensitive)]
    Queen,
}

impl Promotion {
    /// Converts promotion to the unerlying piece.
    pub fn piece(self) -> Piece {
        match self {
            Promotion::Bishop => Piece::Bishop,
            Promotion::Knight => Piece::Knight,
            Promotion::Rook => Piece::Rook,
            Promotion::Queen => Piece::Queen,
        }
    }
}
