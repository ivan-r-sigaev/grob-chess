use crate::bitboard::Rank;
use std::{fmt, ops::Not};
use strum::{EnumCount, EnumIter, FromRepr, VariantArray};

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumCount, EnumIter, VariantArray, FromRepr)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumCount, EnumIter, VariantArray, FromRepr)]
pub enum Piece {
    Pawn,
    Bishop,
    Knight,
    Rook,
    Queen,
    King,
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
