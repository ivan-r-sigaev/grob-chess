const EMPTY_SET: u64 = 0;
const UNIVERSE_SET: u64 = !EMPTY_SET;

const A_FILE: u64 = 0x101010101010101;
//const B_FILE: u64 = 0x0202020202020202;
const H_FILE: u64 = 0x8080808080808080;
const NOT_A_FILE: u64 = !A_FILE;
const NOT_H_FILE: u64 = !H_FILE;

const DIA_A1_H8: u64 = 0x8040201008040201;
const DIA_H1_A8: u64 = 0x0102040810204080;

//const DIA_C2_H7: u64 = 0x0080402010080400;

const RANK1: u64 = 0xff;
/*const RANK2: u64 = 0xff00;
const RANK3: u64 = 0xff0000;
const RANK4: u64 = 0xff000000;
const RANK5: u64 = 0xff00000000;
const RANK6: u64 = 0xff0000000000;
const RANK7: u64 = 0xff000000000000;
const RANK8: u64 = 0xff00000000000000;*/

const fn sout_one(bb: u64) -> u64 {
    bb >> 8
}
const fn nort_one(bb: u64) -> u64 {
    bb << 8
}
const fn east_one(bb: u64) -> u64 {
    (bb << 1) & NOT_A_FILE
}
const fn noea_one(bb: u64) -> u64 {
    (bb << 9) & NOT_A_FILE
}
const fn soea_one(bb: u64) -> u64 {
    (bb >> 7) & NOT_A_FILE
}
const fn west_one(bb: u64) -> u64 {
    (bb >> 1) & NOT_H_FILE
}
const fn sowe_one(bb: u64) -> u64 {
    (bb >> 9) & NOT_H_FILE
}
const fn nowe_one(bb: u64) -> u64 {
    (bb << 7) & NOT_H_FILE
}

#[must_use]
pub const fn make_pawn_attack_table() -> [[u64; 64]; 2] {
    let mut result = [[0; 64]; 2];
    let mut i = 0;
    while i < 64 {
        let bb: u64 = 1 << i;
        result[0][i] = nowe_one(bb) | noea_one(bb);
        result[1][i] = sowe_one(bb) | soea_one(bb);
        i += 1;
    }
    result
}

#[must_use]
pub const fn make_knight_attack_table() -> [u64; 64] {
    let mut result = [0; 64];
    let mut i = 0;
    while i < 64 {
        let bb: u64 = 1 << i;
        let h1: u64 = ((bb >> 1) & 0x7f7f7f7f7f7f7f7f) | ((bb << 1) & 0xfefefefefefefefe);
        let h2: u64 = ((bb >> 2) & 0x3f3f3f3f3f3f3f3f) | ((bb << 2) & 0xfcfcfcfcfcfcfcfc);
        result[i] = (h1 << 16) | (h1 >> 16) | (h2 << 8) | (h2 >> 8);
        i += 1;
    }
    result
}

#[must_use]
pub const fn make_king_attack_table() -> [u64; 64] {
    let mut result = [0; 64];
    let mut i = 0;
    while i < 64 {
        let mut bb: u64 = 1 << i;
        let mut attacks: u64 = east_one(bb) | west_one(bb);
        bb |= attacks;
        attacks |= nort_one(bb) | sout_one(bb);
        result[i] = attacks;
        i += 1;
    }
    result
}

const fn rank_mask(sq: i32) -> u64 {
    RANK1 << ((sq as u64) & 56)
}

/*fn file_mask(sq: i32) -> u64 {
    return A_FILE << ((sq as u64) & 7);
}*/
const fn diagonal_mask(sq: i32) -> u64 {
    let diag: i32 = 8 * (sq & 7) - (sq & 56);
    let nort: i32 = -diag & (diag >> 31);
    let sout: i32 = diag & (-diag >> 31);
    (DIA_A1_H8 >> sout) << nort
}
const fn antidiag_mask(sq: i32) -> u64 {
    let diag: i32 = 56 - 8 * (sq & 7) - (sq & 56);
    let nort: i32 = -diag & (diag >> 31);
    let sout: i32 = diag & (-diag >> 31);
    (DIA_H1_A8 >> sout) << nort
}

#[must_use]
pub const fn make_rank_mask_ex_table() -> [u64; 64] {
    let mut result = [0; 64];
    let mut i = 0;
    while i < 64 {
        let bb: u64 = 1 << i;
        result[i] = rank_mask(i as i32) ^ bb;
        i += 1;
    }
    result
}

#[must_use]
pub const fn make_diagonal_mask_ex_table() -> [u64; 64] {
    let mut result = [0; 64];
    let mut i = 0;
    while i < 64 {
        let bb: u64 = 1 << i;
        result[i] = diagonal_mask(i as i32) ^ bb;
        i += 1;
    }
    result
}

#[must_use]
pub const fn make_antidiag_mask_ex_table() -> [u64; 64] {
    let mut result = [0; 64];
    let mut i = 0;
    while i < 64 {
        let bb: u64 = 1 << i;
        result[i] = antidiag_mask(i as i32) ^ bb;
        i += 1;
    }
    result
}

const fn east_attacks(mut rooks: u64, mut empty: u64) -> u64 {
    empty = empty & NOT_A_FILE; // make A-File all occupied, to consider H-A-wraps after shift
    rooks |= empty & (rooks << 1); // 1. fill
    rooks |= empty & (rooks << 1); // 2. fill
    rooks |= empty & (rooks << 1); // 3. fill
    rooks |= empty & (rooks << 1); // 4. fill
    rooks |= empty & (rooks << 1); // 5. fill
    rooks |= empty & (rooks << 1); // 6. fill
    NOT_A_FILE & (rooks << 1)
}
const fn west_attacks(mut rooks: u64, mut empty: u64) -> u64 {
    empty = empty & NOT_H_FILE;
    rooks |= empty & (rooks >> 1);
    rooks |= empty & (rooks >> 1);
    rooks |= empty & (rooks >> 1);
    rooks |= empty & (rooks >> 1);
    rooks |= empty & (rooks >> 1);
    rooks |= empty & (rooks >> 1);
    NOT_H_FILE & (rooks >> 1)
}
/*fn nort_attacks(mut rooks: u64, empty: u64) -> u64 {
    rooks |= empty & (rooks << 8);
    rooks |= empty & (rooks << 8);
    rooks |= empty & (rooks << 8);
    rooks |= empty & (rooks << 8);
    rooks |= empty & (rooks << 8);
    rooks |= empty & (rooks << 8);
    return rooks << 8;
}
fn sout_attacks(mut rooks: u64, empty: u64) -> u64 {
    rooks |= empty & (rooks >> 8);
    rooks |= empty & (rooks >> 8);
    rooks |= empty & (rooks >> 8);
    rooks |= empty & (rooks >> 8);
    rooks |= empty & (rooks >> 8);
    rooks |= empty & (rooks >> 8);
    return rooks >> 8;
}
fn east_occluded(mut rooks: u64, mut empty: u64) -> u64 {
    empty = empty & NOT_A_FILE; // make A-File all occupied, to consider H-A-wraps after shift
    rooks |= empty & (rooks << 1);
    empty = empty & (empty << 1);
    rooks |= empty & (rooks << 2);
    empty = empty & (empty << 2);
    rooks |= empty & (rooks << 4);
    return rooks;
}*/
const fn nort_occluded(mut rooks: u64, mut empty: u64) -> u64 {
    rooks |= empty & (rooks << 8);
    empty = empty & (empty << 8);
    rooks |= empty & (rooks << 16);
    empty = empty & (empty << 16);
    rooks |= empty & (rooks << 32);
    rooks
}

#[must_use]
pub const fn make_kindergarten_fill_up_attacks_table() -> [[u64; 64]; 8] {
    let mut result = [[0; 64]; 8];
    let mut sq = 0;
    while sq < 8 {
        let mut six_bit_occ = 0;
        while six_bit_occ < 64 {
            let rooks: u64 = 1 << sq;
            let empty: u64 = !((six_bit_occ as u64) << 1);
            let first_rank_attacks: u64 = east_attacks(rooks, empty) | west_attacks(rooks, empty);
            result[sq][six_bit_occ] = nort_occluded(first_rank_attacks, UNIVERSE_SET);
            six_bit_occ += 1;
        }
        sq += 1;
    }
    result
}

#[must_use]
pub const fn make_kindergarten_a_file_attacks_table() -> [[u64; 64]; 8] {
    let mut result = [[0; 64]; 8];
    let mut sq = 0;
    while sq < 8 {
        let mut six_bit_occ = 0;
        while six_bit_occ < 64 {
            let rooks: u64 = 1u64 << (7 - sq);
            let empty: u64 = !((six_bit_occ as u64) << 1);
            let first_rank_attacks: u64 = east_attacks(rooks, empty) | west_attacks(rooks, empty);
            let a_file_attack = ((first_rank_attacks.wrapping_mul(DIA_A1_H8)) >> 7) & A_FILE;
            result[sq][six_bit_occ] = a_file_attack;
            six_bit_occ += 1;
        }
        sq += 1;
    }
    result
}

use const_random::const_random;

#[must_use]
pub const fn make_random_u64_table<const SIZE: usize>() -> [u64; SIZE] {
    let mut result = [0; SIZE];
    let mut i = 0;
    while i < SIZE {
        result[i] = const_random!(u64);
        i += 1;
    }
    result
}
