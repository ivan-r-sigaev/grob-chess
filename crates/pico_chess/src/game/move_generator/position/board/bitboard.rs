use std::{ops::{Not, BitAnd, BitOr, BitXor, BitAndAssign, BitOrAssign, BitXorAssign}, mem::transmute};
use pico_chess_proc_macro::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum File {
    A, 
    B, 
    C, 
    D, 
    E, 
    F, 
    G, 
    H
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rank {
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8
}

impl Rank {
    pub fn promotion_row(color: Color) -> Rank {
        return if color == Color::White { Rank::R8 } else { Rank::R1 };
    }
    pub fn pawn_row(color: Color) -> Rank {
        return if color == Color::White { Rank::R2 } else { Rank::R7 };
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Square {
    A1, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8
}

impl Square {
    #[inline(always)]
    pub fn new(file: File, rank: Rank) -> Square {
        unsafe {
            return transmute((rank as u8) * 8 + (file as u8));
        }
    }
    #[inline(always)]
    pub fn after_en_passant(file: File, turn: Color) -> Square {
        return Square::new(
            file, 
            if turn == Color::White { Rank::R6 } else { Rank::R3 }
        );
    }
    #[inline(always)]
    pub fn into_file(self) -> File {
        unsafe {
            return transmute(self as u8 & 7);
        }
    }
    #[inline(always)]
    pub fn into_rank(self) -> Rank {
        unsafe {
            return transmute(self as u8 >> 3);
        }
    }
    #[inline(always)]
    pub fn shifted(self, delta: i8) -> Square {
        unsafe {
            return transmute(((self as i8).wrapping_add(delta)) & 63);
        }
    }
}

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

#[derive(Clone, Copy,  PartialEq, Eq)]
pub struct BitBoard(u64);

impl From<Square> for BitBoard {
    #[inline(always)]
    fn from(value: Square) -> Self {
        return BitBoard(1u64 << (value as u8));
    }
}

impl BitAnd<BitBoard> for BitBoard {
    type Output = BitBoard;

    #[inline(always)]
    fn bitand(self, rhs: BitBoard) -> Self::Output {
        return BitBoard(self.0 & rhs.0);
    }
}

impl BitAndAssign<BitBoard> for BitBoard {
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: BitBoard) {
        self.0 &= rhs.0;
    }
}

impl BitOr<BitBoard> for BitBoard {
    type Output = BitBoard;

    #[inline(always)]
    fn bitor(self, rhs: BitBoard) -> Self::Output {
        return BitBoard(self.0 | rhs.0);
    }
}

impl BitOrAssign<BitBoard> for BitBoard {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: BitBoard) {
        self.0 |= rhs.0;
    }
}

impl BitXor<BitBoard> for BitBoard {
    type Output = BitBoard;

    #[inline(always)]
    fn bitxor(self, rhs: BitBoard) -> Self::Output {
        return BitBoard(self.0 ^ rhs.0);
    }
}

impl BitXorAssign<BitBoard> for BitBoard {
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: BitBoard) {
        self.0 ^= rhs.0;
    }
}

impl Not for BitBoard {
    type Output = BitBoard;

    #[inline(always)]
    fn not(self) -> Self::Output {
        return BitBoard(!self.0);
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

        let next = unsafe {
            self.0.isolated_ls1b().bit_scan_forward_unchecked()
        };
        self.0 = self.0.reset_ls1b();
        return Some(next);
    }
}

impl BitBoard {
    #[inline(always)]
    pub fn serialize(self) -> Serialized {
        return Serialized(self);
    }
}

impl BitBoard {
    pub const EMPTY: BitBoard = BitBoard(0);
    pub const FULL: BitBoard = BitBoard(!0);

    pub const RANK_1: BitBoard = BitBoard(0xff);
    pub const RANK_2: BitBoard = BitBoard(0xff00);
    pub const RANK_3: BitBoard = BitBoard(0xff0000);
    pub const RANK_4: BitBoard = BitBoard(0xff000000);
    pub const RANK_5: BitBoard = BitBoard(0xff00000000);
    pub const RANK_6: BitBoard = BitBoard(0xff0000000000);
    pub const RANK_7: BitBoard = BitBoard(0xff000000000000);
    pub const RANK_8: BitBoard = BitBoard(0xff00000000000000);
}

impl BitBoard {
    #[inline(always)]
    pub fn pawn_attacks(sq: Square, color: Color) -> BitBoard {
        const PAWN_ATTACKS: [[u64; 64]; 2] = make_pawn_attack_table();
        return BitBoard(PAWN_ATTACKS[color as usize][sq as usize]);
    }
    #[inline(always)]
    pub fn knight_attacks(sq: Square) -> BitBoard {
        const KNIGHT_ATTACKS: [u64; 64] = make_knight_attack_table();
        return BitBoard(KNIGHT_ATTACKS[sq as usize]);
    }
    #[inline(always)]
    pub fn king_attacks(sq: Square) -> BitBoard {
        const KING_ATTACKS: [u64; 64] = make_king_attack_table();
        return BitBoard(KING_ATTACKS[sq as usize]);
    }
    #[inline(always)]
    fn fill_up_attacks(mask_ex: &[u64; 64], occ: BitBoard, sq: Square) -> BitBoard {
        const FILL_UP_ATTACKS: [[u64; 64]; 8] = make_kindergarten_fill_up_attacks_table();
        const B_FILE: u64 = 0x0202020202020202;
        let occupance_index = (B_FILE.wrapping_mul(mask_ex[sq as usize] & occ.0) >> 58) as usize;
        return BitBoard(mask_ex[sq as usize] & FILL_UP_ATTACKS[sq.into_file() as usize][occupance_index]);
    }
    #[inline(always)]
    pub fn diagonal_attacks(occ: BitBoard, sq: Square) -> BitBoard {
        const DIAGONAL_MASK_EX: [u64; 64] = make_diagonal_mask_ex_table();
        return BitBoard::fill_up_attacks(&DIAGONAL_MASK_EX, occ, sq);
    }
    #[inline(always)]
    pub fn antidiag_attacks(occ: BitBoard, sq: Square) -> BitBoard {
        const ANTIDIAG_MASK_EX: [u64; 64] = make_antidiag_mask_ex_table();
        return BitBoard::fill_up_attacks(&ANTIDIAG_MASK_EX, occ, sq);
    }
    #[inline(always)]
    pub fn rank_attacks(occ: BitBoard, sq: Square) -> BitBoard {
        const RANK_MASK_EX: [u64; 64] = make_rank_mask_ex_table();
        return BitBoard::fill_up_attacks(&RANK_MASK_EX, occ, sq);
    }
    #[inline(always)]
    pub fn file_attacks(occ: BitBoard, sq: Square) -> BitBoard {
        const A_FILE_ATTACKS: [[u64; 64]; 8] = make_kindergarten_a_file_attacks_table();
        const A_FILE: u64 = 0x101010101010101;
        const DIA_C2_H7: u64 = 0x0080402010080400;
        let occupance_index = ((DIA_C2_H7.wrapping_mul(A_FILE & (occ.0 >> (sq.into_file() as u8)))) >> 58) as usize;
        return BitBoard(A_FILE_ATTACKS[(sq as usize) >> 3][occupance_index] << (sq.into_file() as u8));
    }
    #[inline(always)]
    pub fn bishop_attacks(occ: BitBoard, sq: Square) -> BitBoard {
        return BitBoard::diagonal_attacks(occ, sq) | BitBoard::antidiag_attacks(occ, sq);
    }
    #[inline(always)]
    pub fn rook_attacks(occ: BitBoard, sq: Square) -> BitBoard {
        return BitBoard::file_attacks(occ, sq) | BitBoard::rank_attacks(occ, sq);
    }
    #[inline(always)]
    pub fn queen_attacks(occ: BitBoard, sq: Square) -> BitBoard {
        return BitBoard::bishop_attacks(occ, sq) | BitBoard::rook_attacks(occ, sq);
    }
    #[inline(always)]
    pub fn pawn_pushes(pawns: BitBoard, empty: BitBoard, color: Color) -> BitBoard {
        return BitBoard(((pawns.0 << 8) >> ((color as u8) << 4)) & empty.0);
    }
}

impl BitBoard {
    #[inline(always)]
    pub fn none(self) -> bool {
        return self.0 == 0;
    }
    #[inline(always)]
    pub fn has_square(self, sq: Square) -> bool {
        return !(self & BitBoard::from(sq)).none();
    }
    #[inline(always)]
    pub fn isolated_ls1b(self) -> BitBoard {
        return BitBoard(self.0 & self.0.wrapping_neg());
    }
    #[inline(always)]
    pub fn separated_ls1b(self) -> BitBoard {
        return BitBoard(self.0 ^ self.0.wrapping_sub(1));
    }
    #[inline(always)]
    pub fn reset_ls1b(self) -> BitBoard {
        return BitBoard(self.0 & self.0.wrapping_sub(1));
    }
    #[inline(always)]
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

        return std::mem::transmute(self.isolated_ls1b().0.trailing_zeros() as u8);
    }
    #[inline(always)]
    pub fn bit_scan_forward(self) -> Option<Square> {
        if self.none() {
            return None;
        }
        unsafe {
            return Some(self.bit_scan_forward_unchecked());
        }
    }
}

impl std::fmt::Debug for BitBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut bb = self.0;
        let mut ret = String::new();
        for _y in 0..8 {
            let mut row = String::new();
            for _x in 0..8 {
                if bb & 1 != 0 { row += "1 "; }
                else { row += "_ "; } 
                bb >>= 1;
            }
            ret = row + "\n" + &ret;
        }
        return f.write_str(&ret);
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
