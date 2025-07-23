use std::fmt;

use crate::board::Color;
use bitflags::bitflags;

bitflags! {
    /// Castlight rights of a chess position.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct CastlingRights: u8 {
        /// The castling rights for white kingside castling.
        const WHITE_KING = 1 << 0;
        /// The castling rights for white queenside castling.
        const WHITE_QUEEN = 1 << 1;
        /// The castling rights for black kingside castling.
        const BLACK_KING = 1 << 2;
        /// The castling rights for black queenside castling.
        const BLACK_QUEEN = 1 << 3;
    }
}

impl CastlingRights {
    /// Constructs the king's castling rights for the given color.
    ///
    /// # Arguments
    /// * `color`: Color
    ///
    /// # Returns
    /// `Self` - the king's castling rights for the given color
    #[inline(always)]
    #[must_use]
    pub fn kingside(color: Color) -> Self {
        if color == Color::White {
            Self::WHITE_KING
        } else {
            Self::BLACK_KING
        }
    }

    /// Constructs the queen's castling rights for the given color.
    ///
    /// # Arguments
    /// * `color`: Color
    ///
    /// # Returns
    /// `Self` - the queen's castling rights for the given color
    #[inline(always)]
    #[must_use]
    pub fn queenside(color: Color) -> Self {
        if color == Color::White {
            Self::WHITE_QUEEN
        } else {
            Self::BLACK_QUEEN
        }
    }

    /// Constructs the castling rights for the given color.
    ///
    /// # Arguments
    /// * `color`: Color
    ///
    /// # Returns
    /// `Self` - the castling rights for the given color
    ///
    /// # Examples
    /// ```rust
    /// use position::prelude::{CastlingRights, Color};
    ///
    /// let white = CastlingRights::both_sides(Color::White);
    /// let white_king = CastlingRights::WHITE_KING;
    /// let white_queen = CastlingRights::WHITE_QUEEN;
    ///
    /// assert_eq!(white, white_king | white_queen);
    ///
    /// let black = CastlingRights::both_sides(Color::Black);
    /// let black_king = CastlingRights::BLACK_KING;
    /// let black_queen = CastlingRights::BLACK_QUEEN;
    ///
    /// assert_eq!(black, black_king | black_queen);
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn both_sides(color: Color) -> Self {
        if color == Color::White {
            Self::WHITE_QUEEN | Self::WHITE_KING
        } else {
            Self::BLACK_QUEEN | Self::BLACK_KING
        }
    }
}

impl fmt::Display for CastlingRights {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}{}{}",
            if self.contains(Self::WHITE_KING) {
                "K"
            } else {
                ""
            },
            if self.contains(Self::WHITE_QUEEN) {
                "Q"
            } else {
                ""
            },
            if self.contains(Self::BLACK_KING) {
                "k"
            } else {
                ""
            },
            if self.contains(Self::WHITE_QUEEN) {
                "q"
            } else {
                ""
            }
        )
    }
}
