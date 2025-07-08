use std::mem::transmute;
use strum::{EnumCount, VariantArray};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumCount, VariantArray)]
pub enum File {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumCount, VariantArray)]
pub enum Rank {
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
}

#[repr(i8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumCount, VariantArray)]
pub enum PosDiag {
    H1H1 = -7,
    G1H2,
    F1H3,
    E1H4,
    D1H5,
    C1H6,
    B1H7,
    A1H8,
    A2G8,
    A3F8,
    A4E8,
    A5D8,
    A6C8,
    A7B8,
    A8A8,
}

#[repr(i8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumCount, VariantArray)]
pub enum NegDiag {
    A1A1 = -7,
    A2B1,
    A3C1,
    A4D1,
    A5E1,
    A6F1,
    A7G1,
    A8H1,
    B8H2,
    C8H3,
    D8H4,
    E8H5,
    F8H6,
    G8H7,
    H8H8,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumCount, VariantArray)]
pub enum Square {
    A1,
    B1,
    C1,
    D1,
    E1,
    F1,
    G1,
    H1,
    A2,
    B2,
    C2,
    D2,
    E2,
    F2,
    G2,
    H2,
    A3,
    B3,
    C3,
    D3,
    E3,
    F3,
    G3,
    H3,
    A4,
    B4,
    C4,
    D4,
    E4,
    F4,
    G4,
    H4,
    A5,
    B5,
    C5,
    D5,
    E5,
    F5,
    G5,
    H5,
    A6,
    B6,
    C6,
    D6,
    E6,
    F6,
    G6,
    H6,
    A7,
    B7,
    C7,
    D7,
    E7,
    F7,
    G7,
    H7,
    A8,
    B8,
    C8,
    D8,
    E8,
    F8,
    G8,
    H8,
}

impl Square {
    #[inline(always)]
    #[must_use]
    pub fn new(file: File, rank: Rank) -> Square {
        unsafe { transmute((rank as u8) * 8 + (file as u8)) }
    }
    #[inline(always)]
    #[must_use]
    pub fn into_file(self) -> File {
        unsafe { transmute(self as u8 & 7) }
    }
    #[inline(always)]
    #[must_use]
    pub fn into_rank(self) -> Rank {
        unsafe { transmute(self as u8 >> 3) }
    }
    #[inline(always)]
    #[must_use]
    pub fn shifted(self, delta: i8) -> Square {
        unsafe { transmute(((self as i8).wrapping_add(delta)) & 63) }
    }
}
