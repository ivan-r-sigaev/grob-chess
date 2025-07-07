use std::{ops::Not, mem::transmute};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    White,
    Black
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