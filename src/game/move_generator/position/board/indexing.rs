use crate::game::position::board::bitboard::Rank;
use std::{mem::transmute, ops::Not};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
        unsafe { transmute((self as u8) ^ 0x01) }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Piece {
    Pawn,
    Bishop,
    Knight,
    Rook,
    Queen,
    King,
}
