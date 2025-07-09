use strum::{EnumCount, FromRepr, VariantArray};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumCount, VariantArray, FromRepr)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumCount, VariantArray, FromRepr)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumCount, VariantArray, FromRepr)]
pub enum PosDiag {
    H1H1 = -(Rank::COUNT as i8) + 1,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumCount, VariantArray, FromRepr)]
pub enum NegDiag {
    A1A1 = -(Rank::COUNT as i8) + 1,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumCount, VariantArray, FromRepr)]
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
    pub const fn straights(rank: Rank, file: File) -> Square {
        Self::from_repr(rank as u8 * File::COUNT as u8 + file as u8).unwrap()
    }
    #[inline(always)]
    #[must_use]
    pub const fn diagonals(positive: PosDiag, negative: NegDiag) -> Square {
        let rank = Rank::from_repr(((positive as i8 + negative as i8) / 2) as u8).unwrap();
        let file = File::from_repr(((negative as i8 - positive as i8) / 2) as u8).unwrap();
        Self::straights(rank, file)
    }
    #[inline(always)]
    #[must_use]
    pub const fn file(self) -> File {
        File::from_repr(self as u8 % File::COUNT as u8).unwrap()
    }
    #[inline(always)]
    #[must_use]
    pub const fn rank(self) -> Rank {
        Rank::from_repr(self as u8 / File::COUNT as u8).unwrap()
    }
    #[inline(always)]
    #[must_use]
    pub const fn pos_diag(self) -> PosDiag {
        PosDiag::from_repr(self.rank() as i8 - self.file() as i8).unwrap()
    }
    #[inline(always)]
    #[must_use]
    pub const fn neg_diag(self) -> NegDiag {
        NegDiag::from_repr(self.rank() as i8 + self.file() as i8 - (Rank::COUNT as i8) + 1).unwrap()
    }
    #[inline(always)]
    #[must_use]
    pub const fn shifted(self, delta: i8) -> Square {
        Self::from_repr(
            (self as i8)
                .wrapping_add(delta)
                .rem_euclid(Square::COUNT as i8) as u8,
        )
        .unwrap()
    }
}
