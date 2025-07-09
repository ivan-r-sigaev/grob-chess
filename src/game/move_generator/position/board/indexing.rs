use crate::game::position::board::bitboard::Rank;
use std::ops::Not;
use strum::{EnumCount, FromRepr, VariantArray};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumCount, VariantArray, FromRepr)]
pub enum Color {
    White,
    Black,
}

impl Color {
    #[must_use]
    pub fn promotion_rank(self) -> Rank {
        if self == Color::White {
            Rank::R8
        } else {
            Rank::R1
        }
    }
    #[must_use]
    pub fn pawn_rank(self) -> Rank {
        if self == Color::White {
            Rank::R2
        } else {
            Rank::R7
        }
    }
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

    #[inline(always)]
    fn not(self) -> Self::Output {
        Self::from_repr((self as u8 + 1) % 2).unwrap()
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumCount, VariantArray, FromRepr)]
pub enum Piece {
    Pawn,
    Bishop,
    Knight,
    Rook,
    Queen,
    King,
}
