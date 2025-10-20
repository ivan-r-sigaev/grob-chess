use bitflags::bitflags;
use std::fmt;

use crate::pieces::Color;

bitflags! {
    /// Castlight rights of a chess position.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct CastlingRights: u8 {
        /// White kingside castling.
        const WHITE_KING = 1 << 0;
        /// White queenside castling.
        const WHITE_QUEEN = 1 << 1;
        /// Black kingside castling.
        const BLACK_KING = 1 << 2;
        /// Black queenside castling.
        const BLACK_QUEEN = 1 << 3;
    }
}

impl CastlingRights {
    /// Returns the kingside castling rights for the given color.
    #[inline(always)]
    #[must_use]
    pub fn kingside(color: Color) -> Self {
        if color == Color::White {
            Self::WHITE_KING
        } else {
            Self::BLACK_KING
        }
    }
    /// Returns the queenside castling rights for the given color.
    #[inline(always)]
    #[must_use]
    pub fn queenside(color: Color) -> Self {
        if color == Color::White {
            Self::WHITE_QUEEN
        } else {
            Self::BLACK_QUEEN
        }
    }
    /// Returns the full castling rights for the given color.
    ///
    /// # Examples
    /// ```rust
    /// use grob_core::castling_rights::CastlingRights;
    /// use grob_core::pieces::Color;
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
    /// Constructs new castling rights from FEN notation segment.
    pub fn from_fen_segment(fen: &str) -> Option<Self> {
        let mut res = Self::empty();
        if fen == "-" {
            return Some(res);
        }

        let mut chars = fen.chars().peekable();
        if chars.peek() == Some(&'K') {
            res |= Self::WHITE_KING;
            chars.next();
        }
        if chars.peek() == Some(&'Q') {
            res |= Self::WHITE_QUEEN;
            chars.next();
        }
        if chars.peek() == Some(&'k') {
            res |= Self::BLACK_KING;
            chars.next();
        }
        if chars.peek() == Some(&'q') {
            res |= Self::BLACK_QUEEN;
            chars.next();
        }
        if chars.peek().is_none() {
            Some(res)
        } else {
            None
        }
    }
    /// Constructs FEN notation segment for castling rights.
    pub fn into_fen_segment(self) -> String {
        if self.is_empty() {
            return String::from("-");
        }
        format!(
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
            if self.contains(Self::BLACK_QUEEN) {
                "q"
            } else {
                ""
            },
        )
    }
}

impl fmt::Display for CastlingRights {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.into_fen_segment())
    }
}
