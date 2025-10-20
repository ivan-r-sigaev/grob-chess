use std::ops::Not;
use strum::{EnumCount, EnumIter, FromRepr, VariantArray};

use crate::square::{Rank, Square};

/// Color of a chess piece.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumCount, EnumIter, VariantArray, FromRepr, Hash)]
pub enum Color {
    /// White pieces.
    White,
    /// Black pieces.
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
    /// use grob_core::Color;
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumCount, EnumIter, VariantArray, FromRepr, Hash)]
pub enum Piece {
    /// Pawn pieces.
    Pawn,
    /// Knight pieces.
    Knight,
    /// Bishop pieces.
    Bishop,
    /// Rook pieces.
    Rook,
    /// Queen pieces.
    Queen,
    /// King pieces.
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumCount, EnumIter, VariantArray, FromRepr, Hash)]
pub enum Promotion {
    /// Promotion to bishop.
    Bishop,
    /// Promotion to knight.
    Knight,
    /// Promotion to rook.
    Rook,
    /// Promotion to queen.
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

/// Converts the piece in FEN notation to color/piece kind combintion.
pub fn piece_from_fen(ch: char) -> Option<(Color, Piece)> {
    let piece = match ch.to_ascii_lowercase() {
        'p' => Piece::Pawn,
        'n' => Piece::Knight,
        'b' => Piece::Bishop,
        'r' => Piece::Rook,
        'q' => Piece::Queen,
        'k' => Piece::King,
        _ => return None,
    };
    let color = match ch.is_ascii_lowercase() {
        true => Color::Black,
        false => Color::White,
    };
    Some((color, piece))
}

/// Converts the color and piece kind to FEN piece.
pub fn piece_to_fen(color: Color, piece: Piece) -> char {
    let letter = match piece {
        Piece::Pawn => 'p',
        Piece::Knight => 'k',
        Piece::Bishop => 'b',
        Piece::Rook => 'r',
        Piece::Queen => 'q',
        Piece::King => 'k',
    };
    if color == Color::White {
        letter.to_ascii_uppercase()
    } else {
        letter
    }
}
