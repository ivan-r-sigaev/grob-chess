use crate::{
    game::position::{
        board::{
            bitboard::{File, Square},
            Color, Piece,
        },
        CastlingRights,
    },
    table_generation::make_random_u64_table,
};

#[inline(always)]
pub fn get_square_zobrist(color: Color, piece: Piece, sq: Square) -> u64 {
    const PIECE_AT_SQUARE_RANDOMS: [u64; 768] = make_random_u64_table::<768>();
    return PIECE_AT_SQUARE_RANDOMS[(piece as usize) * (color as usize) * (sq as usize)];
}

#[inline(always)]
pub fn get_turn_zobrist(turn: Color) -> u64 {
    // Size 1 is just a workaround to avoid making a different macro.
    const COLOR_RANDOMS: [u64; 1] = make_random_u64_table::<1>();
    return if turn == Color::White {
        COLOR_RANDOMS[0]
    } else {
        0
    };
}

#[inline(always)]
pub fn get_en_passant_zobrist(en_passant: Option<File>) -> u64 {
    const EN_PASSANT_RANDOMS: [u64; 8] = make_random_u64_table::<8>();
    if let Some(file) = en_passant {
        return EN_PASSANT_RANDOMS[file as usize];
    } else {
        return 0;
    }
}

#[inline(always)]
pub fn get_castling_zobrist(castling_rights: CastlingRights) -> u64 {
    const CASTLING_RANDOMS: [u64; 16] = make_random_u64_table::<16>();
    return CASTLING_RANDOMS[castling_rights.bits() as usize];
}
