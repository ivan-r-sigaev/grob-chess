use crate::board::Rank;
use std::{fmt, ops::Not, str::FromStr};
use strum::{EnumCount, EnumIter, FromRepr, VariantArray};

/// Color of a chess piece.
///
/// # Examples
/// ```rust
/// use position::board::Color;
/// use strum::FromRepr;
///
/// assert_eq!(Color::from_repr(0), Some(Color::White));
/// assert_eq!(Color::from_repr(1), Some(Color::Black));
/// ```
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumCount, EnumIter, VariantArray, FromRepr, Hash)]
pub enum Color {
    /// White pieces.
    White,
    /// Black pieces.
    Black,
}

impl Color {
    /// Returns the rank where the pawns of this color can promote.
    ///
    /// # Examples
    /// ```rust
    /// use position::board::{Color, Rank};
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
    /// # Examples
    /// ```rust
    /// use position::board::{Color, Rank};
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

    /// Returns the rank where the pawns of this color will be after performing en passant.
    ///
    /// # Examples
    /// ```rust
    /// use position::board::{Color, Rank};
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
    /// # Examples
    /// ```rust
    /// use position::board::Color;
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

/// Chess piece type.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumCount, EnumIter, VariantArray, FromRepr, Hash)]
pub enum Piece {
    /// Pawn pieces.
    Pawn,
    /// Bishop pieces.
    Bishop,
    /// Knight pieces.
    Knight,
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
