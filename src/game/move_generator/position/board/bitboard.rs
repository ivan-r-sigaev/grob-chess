use super::Color;
use crate::table_generation::{
    make_antidiag_mask_ex_table, make_diagonal_mask_ex_table,
    make_kindergarten_a_file_attacks_table, make_kindergarten_fill_up_attacks_table,
    make_king_attack_table, make_knight_attack_table, make_pawn_attack_table,
    make_rank_mask_ex_table,
};
pub use indexing::{File, Rank, Square};
use std::ops::{
    BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl, ShlAssign, Shr,
    ShrAssign,
};
use strum::EnumCount;

mod indexing;

#[derive(Clone, Copy, PartialEq, Eq)]
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
        RANK_ONE.shl(File::COUNT as u8 + value as u8)
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
    pub const fn shl(self, rhs: u8) -> Self {
        BitBoard(self.0 << rhs)
    }
    #[inline(always)]
    #[must_use]
    pub const fn shr(self, rhs: u8) -> Self {
        BitBoard(self.0 >> rhs)
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
    pub const fn shl_assign(&mut self, rhs: u8) {
        *self = Self::shl(*self, rhs);
    }
    pub const fn shr_assign(&mut self, rhs: u8) {
        *self = Self::shr(*self, rhs);
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

pub struct Serialized(BitBoard);

impl Iterator for Serialized {
    type Item = Square;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.0.none() {
            return None;
        }

        let next = unsafe { self.0.isolated_ls1b().bit_scan_forward_unchecked() };
        self.0 = self.0.reset_ls1b();
        Some(next)
    }
}

impl BitBoard {
    #[inline(always)]
    #[must_use]
    pub fn serialize(self) -> Serialized {
        Serialized(self)
    }
}

impl BitBoard {
    #[inline(always)]
    #[must_use]
    pub fn pawn_attacks(sq: Square, color: Color) -> BitBoard {
        const PAWN_ATTACKS: [[u64; 64]; 2] = make_pawn_attack_table();
        BitBoard(PAWN_ATTACKS[color as usize][sq as usize])
    }
    #[inline(always)]
    #[must_use]
    pub fn knight_attacks(sq: Square) -> BitBoard {
        const KNIGHT_ATTACKS: [u64; 64] = make_knight_attack_table();
        BitBoard(KNIGHT_ATTACKS[sq as usize])
    }
    #[inline(always)]
    #[must_use]
    pub fn king_attacks(sq: Square) -> BitBoard {
        const KING_ATTACKS: [u64; 64] = make_king_attack_table();
        BitBoard(KING_ATTACKS[sq as usize])
    }
    #[inline(always)]
    #[must_use]
    fn fill_up_attacks(mask_ex: &[u64; 64], occ: BitBoard, sq: Square) -> BitBoard {
        const FILL_UP_ATTACKS: [[u64; 64]; 8] = make_kindergarten_fill_up_attacks_table();
        const B_FILE: u64 = 0x0202020202020202;
        let occupance_index = (B_FILE.wrapping_mul(mask_ex[sq as usize] & occ.0) >> 58) as usize;
        BitBoard(mask_ex[sq as usize] & FILL_UP_ATTACKS[sq.into_file() as usize][occupance_index])
    }
    #[inline(always)]
    #[must_use]
    pub fn diagonal_attacks(occ: BitBoard, sq: Square) -> BitBoard {
        const DIAGONAL_MASK_EX: [u64; 64] = make_diagonal_mask_ex_table();
        BitBoard::fill_up_attacks(&DIAGONAL_MASK_EX, occ, sq)
    }
    #[inline(always)]
    #[must_use]
    pub fn antidiag_attacks(occ: BitBoard, sq: Square) -> BitBoard {
        const ANTIDIAG_MASK_EX: [u64; 64] = make_antidiag_mask_ex_table();
        BitBoard::fill_up_attacks(&ANTIDIAG_MASK_EX, occ, sq)
    }
    #[inline(always)]
    #[must_use]
    pub fn rank_attacks(occ: BitBoard, sq: Square) -> BitBoard {
        const RANK_MASK_EX: [u64; 64] = make_rank_mask_ex_table();
        BitBoard::fill_up_attacks(&RANK_MASK_EX, occ, sq)
    }
    #[inline(always)]
    #[must_use]
    pub fn file_attacks(occ: BitBoard, sq: Square) -> BitBoard {
        const A_FILE_ATTACKS: [[u64; 64]; 8] = make_kindergarten_a_file_attacks_table();
        const A_FILE: u64 = 0x101010101010101;
        const DIA_C2_H7: u64 = 0x0080402010080400;
        let occupance_index =
            ((DIA_C2_H7.wrapping_mul(A_FILE & (occ.0 >> (sq.into_file() as u8)))) >> 58) as usize;
        BitBoard(A_FILE_ATTACKS[(sq as usize) >> 3][occupance_index] << (sq.into_file() as u8))
    }
    #[inline(always)]
    #[must_use]
    pub fn bishop_attacks(occ: BitBoard, sq: Square) -> BitBoard {
        BitBoard::diagonal_attacks(occ, sq) | BitBoard::antidiag_attacks(occ, sq)
    }
    #[inline(always)]
    #[must_use]
    pub fn rook_attacks(occ: BitBoard, sq: Square) -> BitBoard {
        BitBoard::file_attacks(occ, sq) | BitBoard::rank_attacks(occ, sq)
    }
    #[inline(always)]
    #[must_use]
    pub fn queen_attacks(occ: BitBoard, sq: Square) -> BitBoard {
        BitBoard::bishop_attacks(occ, sq) | BitBoard::rook_attacks(occ, sq)
    }
    #[inline(always)]
    #[must_use]
    pub fn pawn_pushes(pawns: BitBoard, empty: BitBoard, color: Color) -> BitBoard {
        BitBoard(((pawns.0 << 8) >> ((color as u8) << 4)) & empty.0)
    }
}

impl BitBoard {
    #[inline(always)]
    #[must_use]
    pub fn none(self) -> bool {
        self.0 == 0
    }
    #[inline(always)]
    #[must_use]
    pub fn has_square(self, sq: Square) -> bool {
        !(self & BitBoard::from(sq)).none()
    }
    #[inline(always)]
    #[must_use]
    pub fn isolated_ls1b(self) -> BitBoard {
        BitBoard(self.0 & self.0.wrapping_neg())
    }
    #[inline(always)]
    #[must_use]
    pub fn separated_ls1b(self) -> BitBoard {
        BitBoard(self.0 ^ self.0.wrapping_sub(1))
    }
    #[inline(always)]
    #[must_use]
    pub fn reset_ls1b(self) -> BitBoard {
        BitBoard(self.0 & self.0.wrapping_sub(1))
    }
    /// # Safety
    /// Call only if `self.none()` is `false`
    #[inline(always)]
    #[must_use]
    pub unsafe fn bit_scan_forward_unchecked(self) -> Square {
        debug_assert!(!self.none());
        // const INDEX64: [u8; 64] = [
        //     0, 47,  1, 56, 48, 27,  2, 60,
        //     57, 49, 41, 37, 28, 16,  3, 61,
        //     54, 58, 35, 52, 50, 42, 21, 44,
        //     38, 32, 29, 23, 17, 11,  4, 62,
        //     46, 55, 26, 59, 40, 36, 15, 53,
        //     34, 51, 20, 43, 31, 22, 10, 45,
        //     25, 39, 14, 33, 19, 30,  9, 24,
        //     13, 18,  8, 12,  7,  6,  5, 63
        // ];
        // const DEBRUIJN64: u64 = 0x03f79d71b4cb0a89;
        // unsafe {
        //     return Some(std::mem::transmute(
        //         INDEX64[((self.separated_ls1b().0.wrapping_mul(DEBRUIJN64)) >> 58) as usize]
        //     ));
        // }

        std::mem::transmute(self.isolated_ls1b().0.trailing_zeros() as u8)
    }
    #[inline(always)]
    #[must_use]
    pub fn bit_scan_forward(self) -> Option<Square> {
        if self.none() {
            return None;
        }
        unsafe { Some(self.bit_scan_forward_unchecked()) }
    }
}

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

// #[inline(always)]
// pub fn get_pawn_attacks(sq: Square, color: Color) -> u64 {
//     const TABLE: [[u64; 64]; 2] = make_pawn_attack_table!();
//     return TABLE[color as usize][sq as usize];
// }

// #[inline(always)]
// pub fn get_knight_attacks(sq: Square) -> u64 {
//     const TABLE: [u64; 64] = make_knight_attack_table!();
//     return TABLE[sq as usize];
// }

// #[inline(always)]
// pub fn get_king_attacks(sq: Square) -> u64 {
//     const TABLE: [u64; 64] = make_king_attack_table!();
//     return TABLE[sq as usize]
// }

// const FILL_UP_ATTACKS: [[u64; 64]; 8] = make_kindergarten_fill_up_attacks_table!();
// const B_FILE: u64 = 0x0202020202020202;

// #[inline(always)]
// pub fn get_diagonal_attacks(mut occ: u64, sq: Square) -> u64 {
//     const DIAGONAL_MASK_EX: [u64; 64] = make_diagonal_mask_ex_table!();
//     occ = B_FILE.wrapping_mul(DIAGONAL_MASK_EX[sq as usize] & occ) >> 58;
//     return DIAGONAL_MASK_EX[sq as usize]
//         & FILL_UP_ATTACKS[(sq as usize) & 7][occ as usize];
// }
// #[inline(always)]
// pub fn get_antidiag_attacks(mut occ: u64, sq: Square) -> u64 {
//     const ANTIDIAG_MASK_EX: [u64; 64] = make_antidiag_mask_ex_table!();
//     occ = B_FILE.wrapping_mul(ANTIDIAG_MASK_EX[sq as usize] & occ) >> 58;
//     return ANTIDIAG_MASK_EX[sq as usize]
//         & FILL_UP_ATTACKS[(sq as usize) & 7][occ as usize];
// }
// #[inline(always)]
// pub fn get_rank_attacks(mut occ: u64, sq: Square) -> u64 {
//     const RANK_MASK_EX: [u64; 64] = make_rank_mask_ex_table!();
//     occ = B_FILE.wrapping_mul(RANK_MASK_EX[sq as usize] & occ) >> 58;
//     return RANK_MASK_EX[sq as usize]
//         & FILL_UP_ATTACKS[(sq as usize) & 7][occ as usize];
// }
// #[inline(always)]
// pub fn get_file_attacks(mut occ: u64, sq: Square) -> u64 {
//     const A_FILE_ATTACKS: [[u64; 64]; 8] = make_kindergarten_a_file_attacks_table!();
//     const A_FILE: u64 = 0x101010101010101;
//     const DIA_C2_H7: u64 = 0x0080402010080400;
//     occ = A_FILE & (occ >> ((sq as u8) & 7));
//     occ = (DIA_C2_H7.wrapping_mul(occ)) >> 58;
//     return A_FILE_ATTACKS[(sq as usize) >> 3][occ as usize] << ((sq as u8) & 7);
// }

// #[inline(always)]
// pub fn get_bishop_attacks(occ: u64, sq: Square) -> u64 {
//     return get_diagonal_attacks(occ, sq) | get_antidiag_attacks(occ, sq);
// }

// #[inline(always)]
// pub fn get_rook_attacks(occ: u64, sq: Square) -> u64 {
//     return get_file_attacks(occ, sq) | get_rank_attacks(occ, sq);
// }

// #[inline(always)]
// pub fn pawn_pushes(pawns: BitBoard, empty: BitBoard, color: Color) -> BitBoard {
//     return BitBoard(((pawns.0 << 8) >> ((color as u8) << 4)) & empty.0);
// }

// #[inline(always)]
// pub fn get_en_passant_square(en_passant: File, turn: Color) -> Square {
//     return Square::new(
//         en_passant,
//         if turn == Color::White { Rank::R6 } else { Rank::R3 }
//     );
// }

// pub fn format_bitboard(mut bb: u64) -> String {
//     let mut ret = String::new();
// 	for _y in 0..8 {
// 		let mut row = String::new();
// 		for _x in 0..8 {
// 			if bb & 1 != 0 { row += "1 "; }
// 			else { row += "_ "; }
//             bb >>= 1;
// 		}
// 		ret = row + "\n" + &ret;
// 	}
// 	return ret;
// }
