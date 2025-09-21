use crate::{CastlingRights, Color, File, Piece, Square};

use const_random::const_random;
use strum::EnumCount;

/// Computes a [zobrist hash] for a chess piece.
///
/// [zobrist hash]: https://www.chessprogramming.org/Zobrist_Hashing
#[must_use]
#[inline(always)]
pub fn get_square_zobrist(color: Color, piece: Piece, sq: Square) -> u64 {
    const TABLE_SIZE: usize = Piece::COUNT * Color::COUNT * Square::COUNT;
    const PIECE_AT_SQUARE_RANDOMS: [u64; TABLE_SIZE] = make_random_u64_table::<TABLE_SIZE>();
    PIECE_AT_SQUARE_RANDOMS[(piece as usize) * (color as usize) * (sq as usize)]
}

/// Computes a [zobrist hash] for the position's turn.
///
/// [zobrist hash]: https://www.chessprogramming.org/Zobrist_Hashing
#[must_use]
#[inline(always)]
pub fn get_turn_zobrist(turn: Color) -> u64 {
    const TABLE_SIZE: usize = Color::COUNT;
    const COLOR_RANDOMS: [u64; TABLE_SIZE] = make_random_u64_table::<TABLE_SIZE>();
    COLOR_RANDOMS[turn as usize]
}

/// Computes a [zobrist hash] for the available en passant of a position.
///
/// [zobrist hash]: https://www.chessprogramming.org/Zobrist_Hashing
#[must_use]
#[inline(always)]
pub fn get_en_passant_zobrist(en_passant: Option<File>) -> u64 {
    const TABLE_SIZE: usize = File::COUNT + 1;
    const EN_PASSANT_RANDOMS: [u64; TABLE_SIZE] = make_random_u64_table::<TABLE_SIZE>();
    match en_passant {
        Some(file) => EN_PASSANT_RANDOMS[file as usize + 1],
        None => EN_PASSANT_RANDOMS[0],
    }
}

/// Computes a [zobrist hash] for the current castling rights of a position.
///
/// [zobrist hash]: https://www.chessprogramming.org/Zobrist_Hashing
#[must_use]
#[inline(always)]
pub fn get_castling_zobrist(castling_rights: CastlingRights) -> u64 {
    const TABLE_SIZE: usize = CastlingRights::all().bits() as usize + 1;
    const CASTLING_RANDOMS: [u64; TABLE_SIZE] = make_random_u64_table::<TABLE_SIZE>();
    CASTLING_RANDOMS[castling_rights.bits() as usize]
}

/// Computes a random table of `u64`s. It is unique each call.
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
