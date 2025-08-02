use crate::indexing::{File, NegDiag, PosDiag, Rank, Square};
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
/// # See Also
/// [bitboard]: https://www.chessprogramming.org/Bitboard_Board-Definition
#[derive(Debug, Clone, Copy)]
pub(crate) struct BitBoard(pub u64);

impl BitBoard {
    /// Empty bitboard.
    pub const EMPTY: BitBoard = BitBoard(0);
    /// Completely filled bitboard.
    pub const FILLED: BitBoard = Self::EMPTY.not();

    /// `const` version of `std::ops::From<Square>::from`.
    ///
    /// Behaves exactly the same as `<Self as From<Square>>::from`.
    #[inline(always)]
    #[must_use]
    pub const fn from_square(value: Square) -> Self {
        const SQUARE_A1: BitBoard = BitBoard(1);
        SQUARE_A1.shl(value as u8)
    }

    /// `const` version of `std::ops::From<Rank>::from`.
    ///
    /// Behaves exactly the same as `<Self as From<Rank>>::from`.
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

    /// `const` version of `std::ops::From<File>::from`.
    ///
    /// Behaves exactly the same as `<Self as From<File>>::from`.
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

    /// `const` version of `std::ops::From<PosDiag>::from`.
    ///
    /// Behaves exactly the same as `<Self as From<PosDiag>>::from`.
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

    /// `const` version of `std::ops::From<NegDiag>::from`.
    ///
    /// Behaves exactly the same as `<Self as From<NegDiag>>::from`.
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

    /// `const` version of `Iterator::next`.
    ///
    /// Behaves exactly the same as `<Self as Iterator>::next`.
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

    /// `const` version of `std::ops::Not::not`.
    ///
    /// Behaves exactly the same as `<Self as Not>::not`.
    #[inline(always)]
    #[must_use]
    pub const fn not(self) -> Self {
        BitBoard(!self.0)
    }

    /// `const` version of `std::ops::BitAnd<Self>::bitand`.
    ///
    /// Behaves exactly the same as `<Self as BitAnd<Self>>::bitand`.
    #[inline(always)]
    #[must_use]
    pub const fn bitand(self, rhs: BitBoard) -> Self {
        BitBoard(self.0 & rhs.0)
    }

    /// `const` version of `std::ops::BitOr<Self>::bitor`.
    ///
    /// Behaves exactly the same as `<Self as BitOr<Self>>::bitor`.
    #[inline(always)]
    #[must_use]
    pub const fn bitor(self, rhs: BitBoard) -> Self {
        BitBoard(self.0 | rhs.0)
    }

    /// `const` version of `std::ops::BitXor<Self>::bitxor`.
    ///
    /// Behaves exactly the same as `<Self as BitXor<Self>>::bitxor`.
    #[inline(always)]
    #[must_use]
    pub const fn bitxor(self, rhs: BitBoard) -> Self {
        BitBoard(self.0 ^ rhs.0)
    }

    /// `const` version of `std::ops::Mul::mul`.
    ///
    /// Behaves exactly the same as `<Self as Mul>::mul`.
    #[inline(always)]
    #[must_use]
    pub const fn mul(self, rhs: Self) -> Self {
        Self(self.0.wrapping_mul(rhs.0))
    }

    /// `const` version of `std::ops::Shl<u8>::shl`.
    ///
    /// Behaves exactly the same as `<Self as Shl<u8>>::shl`.
    #[inline(always)]
    #[must_use]
    pub const fn shl(self, rhs: u8) -> Self {
        BitBoard(self.0 << rhs)
    }

    /// `const` version of `std::ops::Shr<u8>::shr`.
    ///
    /// Behaves exactly the same as `<Self as Shr<u8>>::shr`.
    #[inline(always)]
    #[must_use]
    pub const fn shr(self, rhs: u8) -> Self {
        BitBoard(self.0 >> rhs)
    }

    /// `const` version of `std::ops::PartialEq::eq`.
    ///
    /// Behaves exactly the same as `<Self as PartialEq>::eq`.
    #[inline(always)]
    #[must_use]
    pub const fn eq(&self, rhs: &BitBoard) -> bool {
        self.0 == rhs.0
    }

    /// `const` version of `std::ops::BitAndAssign<Self>::bitand_assign`.
    ///
    /// Behaves exactly the same as `<Self as BitAndAssign<Self>>::bitand_assign`.
    #[inline(always)]
    pub const fn bitand_assign(&mut self, rhs: BitBoard) {
        *self = Self::bitand(*self, rhs);
    }

    /// `const` version of `std::ops::BitOrAssign<Self>::bitor_assign`.
    ///
    /// Behaves exactly the same as `<Self as BitOrAssign<Self>>::bitor_assign`.
    #[inline(always)]
    pub const fn bitor_assign(&mut self, rhs: BitBoard) {
        *self = Self::bitor(*self, rhs);
    }

    /// `const` version of `std::ops::BitXorAssign<Self>::bitxor_assign`.
    ///
    /// Behaves exactly the same as `<Self as BitXorAssign<Self>>::bitxor_assign`.
    #[inline(always)]
    pub const fn bitxor_assign(&mut self, rhs: BitBoard) {
        *self = Self::bitxor(*self, rhs);
    }

    /// `const` version of `std::ops::MulAssign::mul_assign`.
    ///
    /// Behaves exactly the same as `<Self as MulAssign>::mul_assign`.
    #[inline(always)]
    pub const fn mul_assign(&mut self, rhs: BitBoard) {
        *self = Self::mul(*self, rhs);
    }

    /// `const` version of `std::ops::ShlAssign<u8>::shl_assign`.
    ///
    /// Behaves exactly the same as `<Self as ShlAssign<u8>>::shl_assign`.
    #[inline(always)]
    pub const fn shl_assign(&mut self, rhs: u8) {
        *self = Self::shl(*self, rhs);
    }

    /// `const` version of `std::ops::ShrAssign<u8>::shr_assign`.
    ///
    /// Behaves exactly the same as `<Self as ShrAssign<u8>>::shr_assign`.
    #[inline(always)]
    pub const fn shr_assign(&mut self, rhs: u8) {
        *self = Self::shr(*self, rhs);
    }

    /// Checks whether the entire board is unoccupied.
    ///
    /// # Returns
    /// `bool` - whether the entire board is unoccupied
    #[inline(always)]
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Checks if the given square is occupied.
    ///
    /// # Arguments
    /// * `sq` - the square to check
    ///
    /// # Returns
    /// `bool` - whether the given square is occupied
    #[inline(always)]
    #[must_use]
    pub const fn has_square(self, sq: Square) -> bool {
        !self.bitand(BitBoard::from_square(sq)).is_empty()
    }

    /// Tries to get the occupied square with the lowest index, if empty returns `None`.
    ///
    /// # Returns
    /// `Option<Square>`:
    /// - `Some(square: Square)` - the square with the lowest index
    /// - `None` - if the bitboard is empty
    ///
    /// # See Also
    /// [Square] - to see square's indexes
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
    /// # Arguments
    /// * `rhs` - the signed number of bits to shift by
    ///
    /// # Returns
    /// `Self` - the resulting bitboard
    ///
    /// # See Also
    /// [BitBoard::shr]
    /// [BitBoard::shl]
    #[inline(always)]
    #[must_use]
    const fn genshift(self, rhs: i8) -> Self {
        if rhs >= 0 {
            self.shl(rhs as u8)
        } else {
            self.shr(-rhs as u8)
        }
    }

    /// [Isolates] the least signigicant bit of occupancy.
    ///
    /// # Returns
    /// `Self` - the resulting bitboard
    ///
    /// [isolates]: https://www.chessprogramming.org/General_Setwise_Operations#Isolation
    #[inline(always)]
    #[must_use]
    const fn with_isolated_lsb(self) -> BitBoard {
        BitBoard(self.0 & self.0.wrapping_neg())
    }

    /// [Resets] the least signigicant bit of occupancy.
    ///
    /// # Returns
    /// `Self` - the resulting bitboard
    ///
    /// [resets]: https://www.chessprogramming.org/General_Setwise_Operations#Reset
    #[inline(always)]
    #[must_use]
    const fn with_reset_lsb(self) -> BitBoard {
        BitBoard(self.0 & self.0.wrapping_sub(1))
    }
}

impl Iterator for BitBoard {
    type Item = Square;

    /// Returns the occupied square with the smallest index (or `None` if empty) and removes it from the bitboard.
    ///
    /// # Returns
    /// `Option<Square>`:
    /// - `Some(square: Square)` - the occupied square with the smallest index
    /// - `None` - if the bitboard is empty
    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        Self::next(self)
    }
}

impl From<Square> for BitBoard {
    /// Constructs a bitboard with only the given square occupied.
    ///
    /// # Arguments
    /// * `value` - the occupied square
    ///
    /// # Returns
    /// `Self` - the resulting bitboard
    #[inline(always)]
    fn from(value: Square) -> Self {
        Self::from_square(value)
    }
}

impl From<Rank> for BitBoard {
    /// Constructs a bitboard with only the given rank occupied.
    ///
    /// # Arguments
    /// * `value` - the occupied rank
    ///
    /// # Returns
    /// `Self` - the resulting bitboard
    #[inline(always)]
    fn from(value: Rank) -> Self {
        Self::from_rank(value)
    }
}

impl From<File> for BitBoard {
    /// Constructs a bitboard with only the given file occupied.
    ///
    /// # Arguments
    /// * `value` - the occupied file
    ///
    /// # Returns
    /// `Self` - the resulting bitboard
    #[inline(always)]
    fn from(value: File) -> Self {
        Self::from_file(value)
    }
}

impl From<PosDiag> for BitBoard {
    /// Constructs a bitboard with only the given positive diagonal occupied.
    ///
    /// # Arguments
    /// * `diag` - the occupied diagonal
    ///
    /// # Returns
    /// `Self` - the resulting bitboard
    #[inline(always)]
    fn from(value: PosDiag) -> Self {
        Self::from_pos_diag(value)
    }
}

impl From<NegDiag> for BitBoard {
    /// Constructs a bitboard with only the given negative diagonal occupied.
    ///
    /// # Arguments
    /// * `diag` - the occupied diagonal
    ///
    /// # Returns
    /// `Self` - the resulting bitboard
    #[inline(always)]
    fn from(value: NegDiag) -> Self {
        Self::from_neg_diag(value)
    }
}

impl BitAnd<Self> for BitBoard {
    type Output = Self;

    /// Computes the [intersection] (`&`) of two bitboards.
    ///
    /// # Arguments
    /// * rhs - other bitboard
    ///
    /// # Returns
    /// `Self` - the resulting bitboard
    ///
    /// [intersection]: https://www.chessprogramming.org/General_Setwise_Operations#Intersection
    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self::Output {
        Self::bitand(self, rhs)
    }
}

impl BitAndAssign<Self> for BitBoard {
    /// The assigning version of `BitAnd<Self>`.
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: Self) {
        Self::bitand_assign(self, rhs);
    }
}

impl BitOr<Self> for BitBoard {
    type Output = Self;

    /// Computes the [union] (`|`) of two bitboards.
    ///
    /// # Arguments
    /// * rhs - other bitboard
    ///
    /// # Returns
    /// `Self` - the resulting bitboard
    ///
    /// [union]: https://www.chessprogramming.org/General_Setwise_Operations#Union
    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self::bitor(self, rhs)
    }
}

impl BitOrAssign<Self> for BitBoard {
    /// The assigning version of `BitOr<Self>`.
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: Self) {
        Self::bitor_assign(self, rhs);
    }
}

impl BitXor<Self> for BitBoard {
    type Output = Self;

    /// Computes the [exclusive OR] (`^`) of two bitboards.
    ///
    /// # Arguments
    /// * rhs - other bitboard
    ///
    /// # Returns
    /// `Self` - the resulting bitboard
    ///
    /// [exclusive OR]: https://www.chessprogramming.org/General_Setwise_Operations#Exclusive_Or
    #[inline(always)]
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self::bitxor(self, rhs)
    }
}

impl BitXorAssign<Self> for BitBoard {
    /// The assigning version of `BitXor<Self>`.
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: Self) {
        Self::bitxor_assign(self, rhs);
    }
}

impl Mul for BitBoard {
    type Output = Self;

    /// [Multiplies] the bitboard's occupancies.
    ///
    /// WARNING: THIS IS NOT A TRIVIAL OPERATION. ONLY USE IT IF YOU KNOW WHAT YOU'RE DOING.
    ///
    /// # Arguments
    /// * rhs - other bitboard
    ///
    /// # Returns
    /// `Self` - the resulting bitboard
    ///
    /// [multiplies]: https://www.chessprogramming.org/General_Setwise_Operations#Multiplication
    #[inline(always)]
    fn mul(self, rhs: Self) -> Self::Output {
        Self::mul(self, rhs)
    }
}

impl MulAssign for BitBoard {
    /// The assigning version of `Mul`.
    #[inline(always)]
    fn mul_assign(&mut self, rhs: Self) {
        Self::mul_assign(self, rhs)
    }
}

impl Shl<u8> for BitBoard {
    type Output = Self;

    /// [Bitshifts] the bitboard's occupancy towards higher values.
    ///
    /// # Arguments
    /// * rhs - the number of bits to shift by
    ///
    /// # Returns
    /// `Self` - the resulting bitboard
    ///
    /// [bitshifts]: https://www.chessprogramming.org/General_Setwise_Operations#Shifting_Bitboards
    #[inline(always)]
    fn shl(self, rhs: u8) -> Self::Output {
        Self::shl(self, rhs)
    }
}

impl ShlAssign<u8> for BitBoard {
    /// The assigning version of `Shl<u8>`.
    #[inline(always)]
    fn shl_assign(&mut self, rhs: u8) {
        Self::shl_assign(self, rhs);
    }
}

impl Shr<u8> for BitBoard {
    type Output = Self;

    /// [Bitshifts] the bitboard's occupancy towards lower values.
    ///
    /// # Arguments
    /// * rhs - the number of bits to shift by
    ///
    /// # Returns
    /// `Self` - the resulting bitboard
    ///
    /// [bitshifts]: https://www.chessprogramming.org/General_Setwise_Operations#Shifting_Bitboards
    #[inline(always)]
    fn shr(self, rhs: u8) -> Self::Output {
        Self::shr(self, rhs)
    }
}

impl ShrAssign<u8> for BitBoard {
    /// The assigning version of `Shr<u8>`.
    #[inline(always)]
    fn shr_assign(&mut self, rhs: u8) {
        Self::shr_assign(self, rhs);
    }
}

impl Not for BitBoard {
    type Output = Self;

    /// Returns the [complement set] (`!`) of the bitboard.
    ///
    /// # Returns
    /// `Self` - the resulting bitboard
    ///
    /// [complement set]: https://www.chessprogramming.org/General_Setwise_Operations#Complement_Set
    #[inline(always)]
    fn not(self) -> Self::Output {
        Self::not(self)
    }
}

impl PartialEq for BitBoard {
    /// Checks if two bitboards [are the same].
    ///
    /// # Arguments
    /// * rhs - other bitboard
    ///
    /// # Returns
    /// `bool` - whether the bitboards are the same
    ///
    /// [are the same]: https://www.chessprogramming.org/General_Setwise_Operations#Equality
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
    /// Formats the bitboard for debug purposes.
    ///
    /// The result is a coarse ASCII drawing of the bitboard's occupancy.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
        write!(f, "Bitboard (white side view) {{\n{drawing}}}")
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
