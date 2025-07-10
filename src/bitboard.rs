pub use indexing::{File, NegDiag, PosDiag, Rank, Square};
use std::ops::{
    BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Mul, Not, Shl, ShlAssign, Shr,
    ShrAssign,
};
use strum::EnumCount;

mod indexing;

#[derive(Clone, Copy)]
pub struct BitBoard(pub u64);

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
    #[inline(always)]
    #[must_use]
    pub const fn from_pos_diag(diag: PosDiag) -> Self {
        const DIAG_A1H8: BitBoard = BitBoard::from_square(Square::A1)
            .bitor(BitBoard::from_square(Square::B2))
            .bitor(BitBoard::from_square(Square::C3))
            .bitor(BitBoard::from_square(Square::D4))
            .bitor(BitBoard::from_square(Square::E5))
            .bitor(BitBoard::from_square(Square::F6))
            .bitor(BitBoard::from_square(Square::G7))
            .bitor(BitBoard::from_square(Square::H8));
        DIAG_A1H8.genshift(diag as i8 * File::COUNT as i8)
    }
    #[inline(always)]
    #[must_use]
    pub const fn from_neg_diag(diag: NegDiag) -> Self {
        const DIAG_A8H1: BitBoard = BitBoard::from_square(Square::A8)
            .bitor(BitBoard::from_square(Square::B7))
            .bitor(BitBoard::from_square(Square::C6))
            .bitor(BitBoard::from_square(Square::D5))
            .bitor(BitBoard::from_square(Square::E4))
            .bitor(BitBoard::from_square(Square::F3))
            .bitor(BitBoard::from_square(Square::G2))
            .bitor(BitBoard::from_square(Square::H1));
        DIAG_A8H1.genshift(diag as i8 * File::COUNT as i8)
    }
    #[inline(always)]
    #[must_use]
    pub const fn into_iter(self) -> Self {
        self
    }
    #[inline(always)]
    #[must_use]
    const fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        match self.bit_scan_forward() {
            Some(sq) => {
                *self = self.with_reset_lsb();
                Some(sq)
            }
            None => None,
        }
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
    pub const fn eq(&self, rhs: &BitBoard) -> bool {
        self.0 == rhs.0
    }
    #[inline(always)]
    #[must_use]
    pub const fn ne(&self, rhs: &Self) -> bool {
        !self.eq(rhs)
    }
    #[inline(always)]
    pub const fn bitand_assign(&mut self, rhs: BitBoard) {
        *self = Self::bitand(*self, rhs);
    }
    #[inline(always)]
    pub const fn bitor_assign(&mut self, rhs: BitBoard) {
        *self = Self::bitor(*self, rhs);
    }
    #[inline(always)]
    pub const fn bitxor_assign(&mut self, rhs: BitBoard) {
        *self = Self::bitxor(*self, rhs);
    }
    #[inline(always)]
    pub const fn mul_assign(&mut self, rhs: BitBoard) {
        *self = Self::mul(*self, rhs);
    }
    #[inline(always)]
    pub const fn shl_assign(&mut self, rhs: u8) {
        *self = Self::shl(*self, rhs);
    }
    #[inline(always)]
    pub const fn shr_assign(&mut self, rhs: u8) {
        *self = Self::shr(*self, rhs);
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
    pub const fn bit_scan_forward(self) -> Option<Square> {
        if self.is_empty() {
            return None;
        }

        Some(Square::from_repr(self.with_isolated_lsb().0.trailing_zeros() as u8).unwrap())
    }
    #[inline(always)]
    #[must_use]
    const fn genshift(self, rhs: i8) -> Self {
        if rhs >= 0 {
            self.shl(rhs as u8)
        } else {
            self.shr(-rhs as u8)
        }
    }
    #[inline(always)]
    #[must_use]
    const fn with_isolated_lsb(self) -> BitBoard {
        BitBoard(self.0 & self.0.wrapping_neg())
    }
    #[inline(always)]
    #[must_use]
    const fn with_reset_lsb(self) -> BitBoard {
        BitBoard(self.0 & self.0.wrapping_sub(1))
    }
}

impl Iterator for BitBoard {
    type Item = Square;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        Self::next(self)
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
    #[inline(always)]
    fn from(value: PosDiag) -> Self {
        Self::from_pos_diag(value)
    }
}

impl From<NegDiag> for BitBoard {
    #[inline(always)]
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

    #[inline(always)]
    fn shl(self, rhs: u8) -> Self::Output {
        Self::shl(self, rhs)
    }
}

impl ShlAssign<u8> for BitBoard {
    #[inline(always)]
    fn shl_assign(&mut self, rhs: u8) {
        Self::shl_assign(self, rhs);
    }
}

impl Shr<u8> for BitBoard {
    type Output = BitBoard;

    #[inline(always)]
    fn shr(self, rhs: u8) -> Self::Output {
        Self::shr(self, rhs)
    }
}

impl ShrAssign<u8> for BitBoard {
    #[inline(always)]
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
    #[inline(always)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use strum::VariantArray;

    #[test]
    fn test_rank_composite_bitboard_conversion() {
        for &rank in Rank::VARIANTS {
            let rank_bb = BitBoard::from(rank);
            for &square in Square::VARIANTS {
                let square_bb = BitBoard::from_square(square);
                let is_passed;
                let phrasing;
                if square.rank() == rank {
                    is_passed = rank_bb.has_square(square);
                    phrasing = "does not contain a square on this rank";
                } else {
                    is_passed = !rank_bb.has_square(square);
                    phrasing = "contains a square not on this rank";
                }
                assert!(
                    is_passed,
                    concat!(
                        "Bitboard generated from rank {}!\n",
                        "rank ({:?}):\n{:?}\n",
                        "square ({:?}):\n{:?}\n"
                    ),
                    phrasing, rank, rank_bb, square, square_bb
                )
            }
        }
    }

    #[test]
    fn test_file_composite_bitboard_conversion() {
        for &file in File::VARIANTS {
            let file_bb = BitBoard::from(file);
            for &square in Square::VARIANTS {
                let square_bb = BitBoard::from_square(square);
                let is_passed;
                let phrasing;
                if square.file() == file {
                    is_passed = file_bb.has_square(square);
                    phrasing = "does not contain a square on this file";
                } else {
                    is_passed = !file_bb.has_square(square);
                    phrasing = "contains a square not on this file";
                }
                assert!(
                    is_passed,
                    concat!(
                        "Bitboard generated from file {}!\n",
                        "file ({:?}):\n{:?}\n",
                        "square ({:?}):\n{:?}\n"
                    ),
                    phrasing, file, file_bb, square, square_bb
                )
            }
        }
    }

    #[test]
    fn test_pos_diag_composite_bitboard_conversion() {
        for &diagonal in PosDiag::VARIANTS {
            let diagonal_bb = BitBoard::from(diagonal);
            for &square in Square::VARIANTS {
                let square_bb = BitBoard::from_square(square);
                let is_passed;
                let phrasing;
                if square.pos_diag() == diagonal {
                    is_passed = diagonal_bb.has_square(square);
                    phrasing = "does not contain a square on this diagonal";
                } else {
                    is_passed = !diagonal_bb.has_square(square);
                    phrasing = "contains a square not on this diagonal";
                }
                assert!(
                    is_passed,
                    concat!(
                        "Bitboard generated from positive diagonal {}!\n",
                        "diagonal ({:?}):\n{:?}\n",
                        "square ({:?}):\n{:?}\n"
                    ),
                    phrasing, diagonal, diagonal_bb, square, square_bb
                )
            }
        }
    }

    #[test]
    fn test_neg_diag_composite_bitboard_conversion() {
        for &diagonal in NegDiag::VARIANTS {
            let diagonal_bb = BitBoard::from(diagonal);
            for &square in Square::VARIANTS {
                let square_bb = BitBoard::from_square(square);
                let is_passed;
                let phrasing;
                if square.neg_diag() == diagonal {
                    is_passed = diagonal_bb.has_square(square);
                    phrasing = "does not contain a square on this diagonal";
                } else {
                    is_passed = !diagonal_bb.has_square(square);
                    phrasing = "contains a square not on this diagonal";
                }
                assert!(
                    is_passed,
                    concat!(
                        "Bitboard generated from negative diagonal {}!\n",
                        "diagonal ({:?}):\n{:?}\n",
                        "square ({:?}):\n{:?}\n"
                    ),
                    phrasing, diagonal, diagonal_bb, square, square_bb
                )
            }
        }
    }
}
