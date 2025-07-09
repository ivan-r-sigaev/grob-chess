use crate::game::move_generator::position::board::bitboard::indexing::{NegDiag, PosDiag};
pub use indexing::{File, Rank, Square};
use std::ops::{
    BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Mul, Not, Shl, ShlAssign, Shr,
    ShrAssign,
};
use strum::{EnumCount, VariantArray};

mod indexing;

#[derive(Clone, Copy)]
pub struct BitBoard(u64);

impl BitBoard {
    pub const EMPTY: BitBoard = BitBoard(0);
    pub const FILLED: BitBoard = Self::EMPTY.not();
    #[inline(always)]
    #[must_use]
    pub const fn from_square(value: Square) -> Self {
        const SQUARE_A1: BitBoard = BitBoard(1);
        SQUARE_A1.shl(value as u8)
    }
    #[inline(always)]
    #[must_use]
    pub const fn from_rank(value: Rank) -> Self {
        const RANK_ONE: BitBoard = BitBoard::from_square(Square::A1)
            .bitor(BitBoard::from_square(Square::B1))
            .bitor(BitBoard::from_square(Square::C1))
            .bitor(BitBoard::from_square(Square::D1))
            .bitor(BitBoard::from_square(Square::E1))
            .bitor(BitBoard::from_square(Square::F1))
            .bitor(BitBoard::from_square(Square::G1))
            .bitor(BitBoard::from_square(Square::H1));
        RANK_ONE.shl(File::COUNT as u8 * value as u8)
    }
    #[inline(always)]
    #[must_use]
    pub const fn from_file(value: File) -> Self {
        const FILE_A: BitBoard = BitBoard::from_square(Square::A1)
            .bitor(BitBoard::from_square(Square::A2))
            .bitor(BitBoard::from_square(Square::A3))
            .bitor(BitBoard::from_square(Square::A4))
            .bitor(BitBoard::from_square(Square::A5))
            .bitor(BitBoard::from_square(Square::A6))
            .bitor(BitBoard::from_square(Square::A7))
            .bitor(BitBoard::from_square(Square::A8));
        FILE_A.shl(value as u8)
    }
    pub const fn from_pos_diag(diag: PosDiag) -> Self {
        const DIAG_A1H8: BitBoard = BitBoard::from_square(Square::A1)
            .bitor(BitBoard::from_square(Square::B2))
            .bitor(BitBoard::from_square(Square::C3))
            .bitor(BitBoard::from_square(Square::D4))
            .bitor(BitBoard::from_square(Square::E5))
            .bitor(BitBoard::from_square(Square::F6))
            .bitor(BitBoard::from_square(Square::G7))
            .bitor(BitBoard::from_square(Square::H8));
        DIAG_A1H8.genshift(diag as i8 * 8)
    }
    pub const fn from_neg_diag(diag: NegDiag) -> Self {
        const DIAG_A8H1: BitBoard = BitBoard::from_square(Square::A8)
            .bitor(BitBoard::from_square(Square::B7))
            .bitor(BitBoard::from_square(Square::C6))
            .bitor(BitBoard::from_square(Square::D5))
            .bitor(BitBoard::from_square(Square::E4))
            .bitor(BitBoard::from_square(Square::F3))
            .bitor(BitBoard::from_square(Square::G2))
            .bitor(BitBoard::from_square(Square::H1));
        DIAG_A8H1.genshift(diag as i8 * 8)
    }
    #[inline(always)]
    #[must_use]
    pub const fn not(self) -> Self {
        BitBoard(!self.0)
    }
    #[inline(always)]
    #[must_use]
    pub const fn bitand(self, rhs: BitBoard) -> Self {
        BitBoard(self.0 & rhs.0)
    }
    #[inline(always)]
    #[must_use]
    pub const fn bitor(self, rhs: BitBoard) -> Self {
        BitBoard(self.0 | rhs.0)
    }
    #[inline(always)]
    #[must_use]
    pub const fn bitxor(self, rhs: BitBoard) -> Self {
        BitBoard(self.0 ^ rhs.0)
    }
    #[inline(always)]
    #[must_use]
    pub const fn mul(self, rhs: Self) -> Self {
        Self(self.0.wrapping_mul(rhs.0))
    }
    #[inline(always)]
    #[must_use]
    pub const fn shl(self, rhs: u8) -> Self {
        BitBoard(self.0 << rhs)
    }
    #[inline(always)]
    #[must_use]
    pub const fn shr(self, rhs: u8) -> Self {
        BitBoard(self.0 >> rhs)
    }
    #[inline(always)]
    #[must_use]
    pub const fn genshift(self, rhs: i8) -> Self {
        if rhs >= 0 {
            self.shl(rhs as u8)
        } else {
            self.shr(-rhs as u8)
        }
    }
    #[inline(always)]
    #[must_use]
    pub const fn eq(&self, rhs: &BitBoard) -> bool {
        self.0 == rhs.0
    }
    #[inline(always)]
    #[must_use]
    pub const fn ne(&self, rhs: &Self) -> bool {
        !self.eq(rhs)
    }
    pub const fn bitand_assign(&mut self, rhs: BitBoard) {
        *self = Self::bitand(*self, rhs);
    }
    pub const fn bitor_assign(&mut self, rhs: BitBoard) {
        *self = Self::bitor(*self, rhs);
    }
    pub const fn bitxor_assign(&mut self, rhs: BitBoard) {
        *self = Self::bitxor(*self, rhs);
    }
    pub const fn mul_assign(&mut self, rhs: BitBoard) {
        *self = Self::mul(*self, rhs);
    }
    pub const fn shl_assign(&mut self, rhs: u8) {
        *self = Self::shl(*self, rhs);
    }
    pub const fn shr_assign(&mut self, rhs: u8) {
        *self = Self::shr(*self, rhs);
    }
    #[inline(always)]
    #[must_use]
    pub const fn up(self) -> Self {
        self.shl(File::COUNT as u8)
    }
    #[inline(always)]
    #[must_use]
    pub const fn down(self) -> Self {
        self.shr(File::COUNT as u8)
    }
    #[inline(always)]
    #[must_use]
    pub const fn right(self) -> Self {
        self.bitand(Self::from_file(File::H).not()).shl(1)
    }
    #[inline(always)]
    #[must_use]
    pub const fn left(self) -> Self {
        self.bitand(Self::from_file(File::A).not()).shr(1)
    }
    #[inline(always)]
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
    #[inline(always)]
    #[must_use]
    pub const fn has_square(self, sq: Square) -> bool {
        !self.bitand(BitBoard::from_square(sq)).is_empty()
    }
    #[inline(always)]
    #[must_use]
    pub const fn with_isolated_lsb(self) -> BitBoard {
        BitBoard(self.0 & self.0.wrapping_neg())
    }
    #[inline(always)]
    #[must_use]
    pub const fn with_separated_lsb(self) -> BitBoard {
        BitBoard(self.0 ^ self.0.wrapping_sub(1))
    }
    #[inline(always)]
    #[must_use]
    pub const fn with_reset_lsb(self) -> BitBoard {
        BitBoard(self.0 & self.0.wrapping_sub(1))
    }
    #[inline(always)]
    #[must_use]
    pub const fn bit_scan_forward(self) -> Option<Square> {
        if self.is_empty() {
            return None;
        }

        Some(Square::from_repr(self.with_isolated_lsb().0.trailing_zeros() as u8).unwrap())
    }
    #[inline(always)]
    #[must_use]
    pub const fn serialize(self) -> Serialized {
        Serialized(self)
    }
    #[inline(always)]
    #[must_use]
    pub const fn fill_up(self) -> Self {
        self.mul(Self::from_file(File::A))
    }
    #[inline(always)]
    #[must_use]
    pub const fn attack_right(mut self, occupance: BitBoard) -> Self {
        let empty = occupance.not();
        self.bitor_assign(self.right().bitand(empty)); // 1
        self.bitor_assign(self.right().bitand(empty)); // 2
        self.bitor_assign(self.right().bitand(empty)); // 3
        self.bitor_assign(self.right().bitand(empty)); // 4
        self.bitor_assign(self.right().bitand(empty)); // 5
        self.bitor_assign(self.right().bitand(empty)); // 6
        self.bitand(empty).bitor(self.right()) // 7
    }
    #[inline(always)]
    #[must_use]
    pub const fn attack_left(mut self, occupance: BitBoard) -> Self {
        let empty = occupance.not();
        self.bitor_assign(self.left().bitand(empty)); // 1
        self.bitor_assign(self.left().bitand(empty)); // 2
        self.bitor_assign(self.left().bitand(empty)); // 3
        self.bitor_assign(self.left().bitand(empty)); // 4
        self.bitor_assign(self.left().bitand(empty)); // 5
        self.bitor_assign(self.left().bitand(empty)); // 6
        self.bitand(empty).bitor(self.left()) // 7
    }
    #[inline(always)]
    #[must_use]
    pub const fn attack_up(mut self, occupance: BitBoard) -> Self {
        let empty = occupance.not();
        self.bitor_assign(self.up().bitand(empty)); // 1
        self.bitor_assign(self.up().bitand(empty)); // 2
        self.bitor_assign(self.up().bitand(empty)); // 3
        self.bitor_assign(self.up().bitand(empty)); // 4
        self.bitor_assign(self.up().bitand(empty)); // 5
        self.bitor_assign(self.up().bitand(empty)); // 6
        self.bitand(empty).bitor(self.up()) // 7
    }
    #[inline(always)]
    #[must_use]
    pub const fn attack_down(mut self, occupance: BitBoard) -> Self {
        let empty = occupance.not();
        self.bitor_assign(self.down().bitand(empty)); // 1
        self.bitor_assign(self.down().bitand(empty)); // 2
        self.bitor_assign(self.down().bitand(empty)); // 3
        self.bitor_assign(self.down().bitand(empty)); // 4
        self.bitor_assign(self.down().bitand(empty)); // 5
        self.bitor_assign(self.down().bitand(empty)); // 6
        self.bitand(empty).bitor(self.down()) // 7
    }
    #[inline(always)]
    #[must_use]
    pub const fn rank_to_reversed_file(self) -> Self {
        self.mul(Self::from_pos_diag(PosDiag::A1H8))
            .shr(7)
            .bitand(Self::from_file(File::A))
    }
    #[inline(always)]
    #[must_use]
    pub const fn file_to_reversed_rank(self) -> Self {
        self.mul(Self::from_pos_diag(PosDiag::A1H8)).shr(56)
    }
    #[inline(always)]
    #[must_use]
    pub const fn project_on_rank(self) -> Self {
        self.fill_up().shr(56)
    }
    pub const KINDERGARTEN_OCCUPANCY_MAX: u8 = 64;
    #[inline(always)]
    #[must_use]
    pub const fn into_kindergarten_occupancy(self) -> u8 {
        assert!(self.bitand(BitBoard::from_rank(Rank::R1).not()).is_empty());
        self.bitand(BitBoard::from_square(Square::H1).not())
            .shr(1)
            .0 as u8
    }
    #[inline(always)]
    #[must_use]
    pub const fn from_kindergarten_occupancy_as_rank(file: File, kg_occupancy: u8) -> BitBoard {
        const LOOKUP: [[BitBoard; BitBoard::KINDERGARTEN_OCCUPANCY_MAX as usize]; File::COUNT] = {
            let mut result =
                [[BitBoard::EMPTY; BitBoard::KINDERGARTEN_OCCUPANCY_MAX as usize]; File::COUNT];
            let mut i = 0;
            while i < File::COUNT {
                let file = File::VARIANTS[i];
                let mut kg_occupancy = 0;
                while kg_occupancy < BitBoard::KINDERGARTEN_OCCUPANCY_MAX {
                    let kg_occupancy_bb = BitBoard(kg_occupancy as u64).shl(1);
                    let slider = BitBoard::from_square(Square::straights( Rank::R1, file));
                    result[file as usize][kg_occupancy as usize] = slider
                        .attack_left(kg_occupancy_bb)
                        .bitor(slider.attack_right(kg_occupancy_bb))
                        .fill_up();
                    kg_occupancy += 1;
                }
                i += 1;
            }
            result
        };
        LOOKUP[file as usize][kg_occupancy as usize]
    }
    #[inline(always)]
    #[must_use]
    pub const fn from_kindergarten_occupancy_as_file(rank: Rank, kg_occupancy_rev: u8) -> BitBoard {
        const LOOKUP: [[BitBoard; BitBoard::KINDERGARTEN_OCCUPANCY_MAX as usize]; Rank::COUNT] = {
            let mut result =
                [[BitBoard::EMPTY; BitBoard::KINDERGARTEN_OCCUPANCY_MAX as usize]; Rank::COUNT];
            let mut i = 0;
            while i < Rank::COUNT {
                let rank = Rank::VARIANTS[i];
                let mut kg_occupancy_rev = 0;
                while kg_occupancy_rev < BitBoard::KINDERGARTEN_OCCUPANCY_MAX {
                    let kg_occupancy_rev_bb = BitBoard(kg_occupancy_rev as u64).shl(1);
                    let occupancy_on_a_file = kg_occupancy_rev_bb.rank_to_reversed_file();
                    let slider = BitBoard::from_square(Square::straights(rank, File::A));
                    result[rank as usize][kg_occupancy_rev as usize] = slider
                        .attack_up(occupancy_on_a_file)
                        .bitor(slider.attack_down(occupancy_on_a_file));
                    kg_occupancy_rev += 1;
                }
                i += 1;
            }
            result
        };
        LOOKUP[rank as usize][kg_occupancy_rev as usize]
    }
}

impl From<Square> for BitBoard {
    #[inline(always)]
    fn from(value: Square) -> Self {
        Self::from_square(value)
    }
}

impl From<Rank> for BitBoard {
    #[inline(always)]
    fn from(value: Rank) -> Self {
        Self::from_rank(value)
    }
}

impl From<File> for BitBoard {
    #[inline(always)]
    fn from(value: File) -> Self {
        Self::from_file(value)
    }
}

impl From<PosDiag> for BitBoard {
    fn from(value: PosDiag) -> Self {
        Self::from_pos_diag(value)
    }
}

impl From<NegDiag> for BitBoard {
    fn from(value: NegDiag) -> Self {
        Self::from_neg_diag(value)
    }
}

impl BitAnd<BitBoard> for BitBoard {
    type Output = BitBoard;

    #[inline(always)]
    fn bitand(self, rhs: BitBoard) -> Self::Output {
        Self::bitand(self, rhs)
    }
}

impl BitAndAssign<BitBoard> for BitBoard {
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: BitBoard) {
        Self::bitand_assign(self, rhs);
    }
}

impl BitOr<BitBoard> for BitBoard {
    type Output = BitBoard;

    #[inline(always)]
    fn bitor(self, rhs: BitBoard) -> Self::Output {
        Self::bitor(self, rhs)
    }
}

impl BitOrAssign<BitBoard> for BitBoard {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: BitBoard) {
        Self::bitor_assign(self, rhs);
    }
}

impl BitXor<BitBoard> for BitBoard {
    type Output = BitBoard;

    #[inline(always)]
    fn bitxor(self, rhs: BitBoard) -> Self::Output {
        Self::bitxor(self, rhs)
    }
}

impl BitXorAssign<BitBoard> for BitBoard {
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: BitBoard) {
        Self::bitxor_assign(self, rhs);
    }
}

impl Mul for BitBoard {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: Self) -> Self::Output {
        Self::mul(self, rhs)
    }
}

impl Shl<u8> for BitBoard {
    type Output = BitBoard;

    fn shl(self, rhs: u8) -> Self::Output {
        Self::shl(self, rhs)
    }
}

impl ShlAssign<u8> for BitBoard {
    fn shl_assign(&mut self, rhs: u8) {
        Self::shl_assign(self, rhs);
    }
}

impl Shr<u8> for BitBoard {
    type Output = BitBoard;

    fn shr(self, rhs: u8) -> Self::Output {
        Self::shr(self, rhs)
    }
}

impl ShrAssign<u8> for BitBoard {
    fn shr_assign(&mut self, rhs: u8) {
        Self::shr_assign(self, rhs);
    }
}

impl Not for BitBoard {
    type Output = BitBoard;

    #[inline(always)]
    fn not(self) -> Self::Output {
        Self::not(self)
    }
}

impl PartialEq for BitBoard {
    fn eq(&self, other: &Self) -> bool {
        Self::eq(self, other)
    }
}
impl Eq for BitBoard {}

impl std::fmt::Debug for BitBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut bb = self.0;
        let mut ret = String::new();
        for _y in 0..8 {
            let mut row = String::new();
            for _x in 0..8 {
                if bb & 1 != 0 {
                    row += "1 ";
                } else {
                    row += "_ ";
                }
                bb >>= 1;
            }
            ret = row + "\n" + &ret;
        }
        f.write_str(&ret)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Serialized(BitBoard);

impl Iterator for Serialized {
    type Item = Square;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        match self.0.bit_scan_forward() {
            Some(sq) => {
                self.0 = self.0.with_reset_lsb();
                Some(sq)
            }
            None => None,
        }
    }
}
