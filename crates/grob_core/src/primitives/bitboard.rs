use crate::{File, NegDiag, PosDiag, Rank, Square};
use std::{
    fmt,
    hash::Hash,
    ops::{
        BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Mul, MulAssign, Not, Shl,
        ShlAssign, Shr, ShrAssign,
    },
};
use strum::EnumCount;

/// A [bitboard]. Wraps u64 occupancy mask.
///
/// [bitboard]: https://www.chessprogramming.org/Bitboard_Board-Definition
#[derive(Debug, Clone, Copy)]
pub struct BitBoard(pub u64);

impl BitBoard {
    /// Empty bitboard.
    pub const EMPTY: BitBoard = BitBoard(0);
    /// Fully occupied bitboard.
    pub const FILLED: BitBoard = Self::EMPTY.not();

    /// Constructs a bitboard with only the given square occupied.
    ///
    /// This is a const version of [`From<Square>::from`].
    #[inline(always)]
    #[must_use]
    pub const fn from_square(value: Square) -> Self {
        const SQUARE_A1: BitBoard = BitBoard(1);
        SQUARE_A1.shl(value as u8)
    }

    /// Constructs a bitboard with only the given rank occupied.
    ///
    /// This is a const version of [`From<Rank>::from`].
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

    /// Constructs a bitboard with only the given file occupied.
    ///
    /// This is a const version of [`From<File>::from`].
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

    /// Constructs a bitboard with only the given positive diagonal occupied.
    ///
    /// This is a const version of [`From<PosDiag>::from`].
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

    /// Constructs a bitboard with only the given negative diagonal occupied.
    ///
    /// This is a const version of [`From<NegDiag>::from`].
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

    /// Returns the result of [`BitBoard::bit_scan_forward`] and removes this
    /// square from the bitboard. Returns `None` if the bitboard is empty.
    ///
    /// This is a const version of [`Iterator::next`].
    #[inline(always)]
    #[must_use]
    pub const fn iterator_next(&mut self) -> Option<<Self as Iterator>::Item> {
        match self.bit_scan_forward() {
            Some(sq) => {
                *self = self.with_reset_lsb();
                Some(sq)
            }
            None => None,
        }
    }

    /// Returns the [complement set] (`!`) of the bitboard.
    ///
    /// [complement set]: https://www.chessprogramming.org/General_Setwise_Operations#Complement_Set
    ///
    /// This is a const version of [`Not::not`].
    #[inline(always)]
    #[must_use]
    pub const fn not(self) -> Self {
        BitBoard(!self.0)
    }

    /// Computes the [intersection] (`&`) of two bitboards.
    ///
    /// [intersection]: https://www.chessprogramming.org/General_Setwise_Operations#Intersection
    ///
    /// This is a const version of [`BitAnd::bitand`].
    #[inline(always)]
    #[must_use]
    pub const fn bitand(self, rhs: BitBoard) -> Self {
        BitBoard(self.0 & rhs.0)
    }

    /// Computes the [union] (`|`) of two bitboards.
    ///
    /// [union]: https://www.chessprogramming.org/General_Setwise_Operations#Union
    ///
    /// This is a const version of [`BitOr::bitor`].
    #[inline(always)]
    #[must_use]
    pub const fn bitor(self, rhs: BitBoard) -> Self {
        BitBoard(self.0 | rhs.0)
    }

    /// Computes the [exclusive OR] (`^`) of two bitboards.
    ///
    /// [exclusive OR]: https://www.chessprogramming.org/General_Setwise_Operations#Exclusive_Or
    ///
    /// This is a const version of [`BitXor::bitxor`].
    #[inline(always)]
    #[must_use]
    pub const fn bitxor(self, rhs: BitBoard) -> Self {
        BitBoard(self.0 ^ rhs.0)
    }

    /// [Multiplies] the bitboard's occupancies.
    /// Only use this if you know what you're doing.
    ///
    /// [multiplies]: https://www.chessprogramming.org/General_Setwise_Operations#Multiplication
    ///
    /// This is a const version of [`Mul::mul`].
    #[inline(always)]
    #[must_use]
    pub const fn mul(self, rhs: Self) -> Self {
        Self(self.0.wrapping_mul(rhs.0))
    }

    /// [Bitshifts] the bitboard's occupancy towards higher values.
    ///
    /// [bitshifts]: https://www.chessprogramming.org/General_Setwise_Operations#Shifting_Bitboards
    ///
    /// This is a const version of [`Shl::shl`].
    #[inline(always)]
    #[must_use]
    pub const fn shl(self, rhs: u8) -> Self {
        BitBoard(self.0 << rhs)
    }

    /// [Bitshifts] the bitboard's occupancy towards lower values.
    ///
    /// [bitshifts]: https://www.chessprogramming.org/General_Setwise_Operations#Shifting_Bitboards
    ///
    /// This is a const version of [`Shr::shr`].
    #[inline(always)]
    #[must_use]
    pub const fn shr(self, rhs: u8) -> Self {
        BitBoard(self.0 >> rhs)
    }

    /// Checks if two bitboards [are the same].
    ///
    /// [are the same]: https://www.chessprogramming.org/General_Setwise_Operations#Equality
    ///
    /// This is a const version of [`PartialEq::eq`].
    #[inline(always)]
    #[must_use]
    pub const fn eq(&self, rhs: &BitBoard) -> bool {
        self.0 == rhs.0
    }

    /// The assigning version of [`BitAnd::bitand`].
    ///
    /// This is a const version of [`BitAndAssign::bitand_assign`].
    #[inline(always)]
    pub const fn bitand_assign(&mut self, rhs: BitBoard) {
        *self = Self::bitand(*self, rhs);
    }

    /// The assigning version of [`BitOr::bitor`].
    ///
    /// This is a const version of [`BitOrAssign::bitor_assign`].
    #[inline(always)]
    pub const fn bitor_assign(&mut self, rhs: BitBoard) {
        *self = Self::bitor(*self, rhs);
    }

    /// The assigning version of [`BitXor::bitxor`].
    ///
    /// This is a const version of [`BitXorAssign::bitxor_assign`].
    #[inline(always)]
    pub const fn bitxor_assign(&mut self, rhs: BitBoard) {
        *self = Self::bitxor(*self, rhs);
    }

    /// The assigning version of [`Mul::mul`].
    ///
    /// This is a const version of [`MulAssign::mul_assign`].
    #[inline(always)]
    pub const fn mul_assign(&mut self, rhs: BitBoard) {
        *self = Self::mul(*self, rhs);
    }

    /// The assigning version of [`Shl::shl`].
    ///
    /// This is a const version of [`ShlAssign::shl_assign`].
    #[inline(always)]
    pub const fn shl_assign(&mut self, rhs: u8) {
        *self = Self::shl(*self, rhs);
    }

    /// The assigning version of [`Shr::shr`].
    ///
    /// This is a const version of [`ShrAssign::shr_assign`].
    #[inline(always)]
    pub const fn shr_assign(&mut self, rhs: u8) {
        *self = Self::shr(*self, rhs);
    }

    /// Returns `true` if the bitboard is completely unoccupied.
    #[inline(always)]
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Returns `true` if the given square is occupied.
    #[inline(always)]
    #[must_use]
    pub const fn has_square(self, sq: Square) -> bool {
        !self.bitand(BitBoard::from_square(sq)).is_empty()
    }

    /// Returns the occupied square with the lowest index
    /// or `None` if the bitboard is empty.
    #[inline(always)]
    #[must_use]
    pub const fn bit_scan_forward(self) -> Option<Square> {
        if self.is_empty() {
            return None;
        }

        Some(Square::from_repr(self.with_isolated_lsb().0.trailing_zeros() as u8).unwrap())
    }

    /// Bitshifts the bitboard based on the sign of the input.
    ///
    /// Positive values shift `x` bits to the left,
    /// while negative values shift `|x|` bits to the right.
    ///
    /// # See Also
    /// [BitBoard::shr], [BitBoard::shl] - bitshifting
    #[inline(always)]
    #[must_use]
    pub const fn genshift(self, x: i8) -> Self {
        if x >= 0 {
            self.shl(x as u8)
        } else {
            self.shr(-x as u8)
        }
    }

    /// [Isolates] the least signigicant bit of occupancy.
    ///
    /// [isolates]: https://www.chessprogramming.org/General_Setwise_Operations#Isolation
    #[inline(always)]
    #[must_use]
    pub const fn with_isolated_lsb(self) -> BitBoard {
        BitBoard(self.0 & self.0.wrapping_neg())
    }

    /// [Resets] the least signigicant bit of occupancy.
    ///
    /// [resets]: https://www.chessprogramming.org/General_Setwise_Operations#Reset
    #[inline(always)]
    #[must_use]
    pub const fn with_reset_lsb(self) -> BitBoard {
        BitBoard(self.0 & self.0.wrapping_sub(1))
    }
    /// Creates a string containing an ASCII drawing of the bitboard.
    pub fn image_string(self) -> String {
        let mut bb = self.0;
        let mut drawing = String::new();
        for _y in 0..8 {
            let mut row = String::from("  ");
            for _x in 0..8 {
                if bb & 1 != 0 {
                    row += "1";
                } else {
                    row += "_";
                }
                row += " ";
                bb >>= 1;
            }
            drawing = row + "\n" + &drawing;
        }
        format!("Bitboard (white side view) {{\n{drawing}}}")
    }
}

impl Iterator for BitBoard {
    type Item = Square;

    /// See [`BitBoard::iterator_next`].
    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        Self::iterator_next(self)
    }
}

impl From<Square> for BitBoard {
    /// See [`BitBoard::from_square`].
    #[inline(always)]
    fn from(value: Square) -> Self {
        Self::from_square(value)
    }
}

impl From<Rank> for BitBoard {
    /// See [`BitBoard::from_rank`].
    #[inline(always)]
    fn from(value: Rank) -> Self {
        Self::from_rank(value)
    }
}

impl From<File> for BitBoard {
    /// See [`BitBoard::from_file`].
    #[inline(always)]
    fn from(value: File) -> Self {
        Self::from_file(value)
    }
}

impl From<PosDiag> for BitBoard {
    /// See [`BitBoard::from_pos_diag`].
    #[inline(always)]
    fn from(value: PosDiag) -> Self {
        Self::from_pos_diag(value)
    }
}

impl From<NegDiag> for BitBoard {
    /// See [`BitBoard::from_neg_diag`].
    #[inline(always)]
    fn from(value: NegDiag) -> Self {
        Self::from_neg_diag(value)
    }
}

impl BitAnd<Self> for BitBoard {
    type Output = Self;

    /// See [`BitBoard::bitand`].
    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self::Output {
        Self::bitand(self, rhs)
    }
}

impl BitAndAssign<Self> for BitBoard {
    /// See [`BitBoard::bitand_assign`].
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: Self) {
        Self::bitand_assign(self, rhs);
    }
}

impl BitOr<Self> for BitBoard {
    type Output = Self;

    /// See [`BitBoard::bitor`].
    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self::bitor(self, rhs)
    }
}

impl BitOrAssign<Self> for BitBoard {
    /// See [`BitBoard::bitor_assign`].
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: Self) {
        Self::bitor_assign(self, rhs);
    }
}

impl BitXor<Self> for BitBoard {
    type Output = Self;

    /// See [`BitBoard::bitxor`].
    #[inline(always)]
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self::bitxor(self, rhs)
    }
}

impl BitXorAssign<Self> for BitBoard {
    /// See [`BitBoard::bitxor_assign`].
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: Self) {
        Self::bitxor_assign(self, rhs);
    }
}

impl Mul for BitBoard {
    type Output = Self;

    /// See [`BitBoard::mul`].
    #[inline(always)]
    fn mul(self, rhs: Self) -> Self::Output {
        Self::mul(self, rhs)
    }
}

impl MulAssign for BitBoard {
    /// See [`BitBoard::mul_assign`].
    #[inline(always)]
    fn mul_assign(&mut self, rhs: Self) {
        Self::mul_assign(self, rhs)
    }
}

impl Shl<u8> for BitBoard {
    type Output = Self;

    /// See [`BitBoard::shl`].
    #[inline(always)]
    fn shl(self, rhs: u8) -> Self::Output {
        Self::shl(self, rhs)
    }
}

impl ShlAssign<u8> for BitBoard {
    /// See [`BitBoard::shl_assign`].
    #[inline(always)]
    fn shl_assign(&mut self, rhs: u8) {
        Self::shl_assign(self, rhs);
    }
}

impl Shr<u8> for BitBoard {
    type Output = Self;

    /// See [`BitBoard::shr`].
    #[inline(always)]
    fn shr(self, rhs: u8) -> Self::Output {
        Self::shr(self, rhs)
    }
}

impl ShrAssign<u8> for BitBoard {
    /// See [`BitBoard::shr_assign`].
    #[inline(always)]
    fn shr_assign(&mut self, rhs: u8) {
        Self::shr_assign(self, rhs);
    }
}

impl Not for BitBoard {
    type Output = Self;

    /// See [`BitBoard::not`].
    #[inline(always)]
    fn not(self) -> Self::Output {
        Self::not(self)
    }
}

impl PartialEq for BitBoard {
    /// See [`BitBoard::eq`].
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        Self::eq(self, other)
    }
}
impl Eq for BitBoard {}

impl Hash for BitBoard {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl fmt::Display for BitBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.image_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    #[test]
    fn test_rank_composite_bitboard_conversion() {
        for rank in Rank::iter() {
            let rank_bb = BitBoard::from(rank);
            for square in Square::iter() {
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
                        "rank ({}):\n{}\n",
                        "square ({}):\n{}\n"
                    ),
                    phrasing, rank, rank_bb, square, square_bb
                )
            }
        }
    }

    #[test]
    fn test_file_composite_bitboard_conversion() {
        for file in File::iter() {
            let file_bb = BitBoard::from(file);
            for square in Square::iter() {
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
                        "file ({}):\n{}\n",
                        "square ({}):\n{}\n"
                    ),
                    phrasing, file, file_bb, square, square_bb
                )
            }
        }
    }

    #[test]
    fn test_pos_diag_composite_bitboard_conversion() {
        for diagonal in PosDiag::iter() {
            let diagonal_bb = BitBoard::from(diagonal);
            for square in Square::iter() {
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
                        "diagonal ({}):\n{}\n",
                        "square ({}):\n{}\n"
                    ),
                    phrasing, diagonal, diagonal_bb, square, square_bb
                )
            }
        }
    }

    #[test]
    fn test_neg_diag_composite_bitboard_conversion() {
        for diagonal in NegDiag::iter() {
            let diagonal_bb = BitBoard::from(diagonal);
            for square in Square::iter() {
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
                        "diagonal ({}):\n{}\n",
                        "square ({}):\n{}\n"
                    ),
                    phrasing, diagonal, diagonal_bb, square, square_bb
                )
            }
        }
    }

    #[test]
    fn test_bitscan() {
        assert_eq!(BitBoard::EMPTY.bit_scan_forward(), None);
        assert_eq!(BitBoard::FILLED.bit_scan_forward(), Some(Square::A1));
    }
}
