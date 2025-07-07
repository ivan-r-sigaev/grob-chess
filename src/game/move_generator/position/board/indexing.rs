use std::{ops::Not, mem::transmute};
use crate::game::position::board::bitboard::Rank;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    White,
    Black
}

impl Color {
    pub fn promotion_rank(self) -> Rank {
        return if self == Color::White { Rank::R8 } else { Rank::R1 };
    }
    pub fn pawn_rank(self) -> Rank {
        return if self == Color::White { Rank::R2 } else { Rank::R7 };
    }
}

impl Not for Color {
    type Output = Self;

    #[inline(always)]
    fn not(self) -> Self::Output {
        unsafe {
            return transmute((self as u8) ^ 0x01);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Piece {
    Pawn,
    Bishop,
    Knight,
    Rook,
    Queen,
    King
}