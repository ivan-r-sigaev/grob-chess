use crate::bitboard::{File, Square};
use crate::board::{Color, Piece};
use crate::position::CastlingRights;

use const_random::const_random;

#[must_use]
const fn make_random_u64_table<const SIZE: usize>() -> [u64; SIZE] {
    let mut result = [0; SIZE];
    let mut i = 0;
    while i < SIZE {
        result[i] = const_random!(u64);
        i += 1;
    }
    result
}

#[must_use]
#[inline(always)]
pub fn get_square_zobrist(color: Color, piece: Piece, sq: Square) -> u64 {
    const PIECE_AT_SQUARE_RANDOMS: [u64; 768] = make_random_u64_table::<768>();
    PIECE_AT_SQUARE_RANDOMS[(piece as usize) * (color as usize) * (sq as usize)]
}

#[must_use]
#[inline(always)]
pub fn get_turn_zobrist(turn: Color) -> u64 {
    // Size 1 is just a workaround to avoid making a different macro.
    const COLOR_RANDOMS: [u64; 1] = make_random_u64_table::<1>();
    if turn == Color::White {
        COLOR_RANDOMS[0]
    } else {
        0
    }
}

#[must_use]
#[inline(always)]
pub fn get_en_passant_zobrist(en_passant: Option<File>) -> u64 {
    const EN_PASSANT_RANDOMS: [u64; 8] = make_random_u64_table::<8>();
    if let Some(file) = en_passant {
        EN_PASSANT_RANDOMS[file as usize]
    } else {
        0
    }
}

#[must_use]
#[inline(always)]
pub fn get_castling_zobrist(castling_rights: CastlingRights) -> u64 {
    const CASTLING_RANDOMS: [u64; 16] = make_random_u64_table::<16>();
    CASTLING_RANDOMS[castling_rights.bits() as usize]
}
