use proc_macro::TokenStream;
use std::fmt::Write;

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

fn sout_one(bb: u64) -> u64 { return  bb >> 8; }
fn nort_one(bb: u64) -> u64 { return  bb << 8; }
fn east_one(bb: u64) -> u64 { return (bb << 1) & NOT_A_FILE; }
fn noea_one(bb: u64) -> u64 { return (bb << 9) & NOT_A_FILE; }
fn soea_one(bb: u64) -> u64 { return (bb >> 7) & NOT_A_FILE; }
fn west_one(bb: u64) -> u64 { return (bb >> 1) & NOT_H_FILE; }
fn sowe_one(bb: u64) -> u64 { return (bb >> 9) & NOT_H_FILE; }
fn nowe_one(bb: u64) -> u64 { return (bb << 7) & NOT_H_FILE; }


#[proc_macro]
pub fn make_pawn_attack_table(_item: TokenStream) -> TokenStream {
    assert!(_item.is_empty());
    let mut white = String::new();
    let mut black = String::new();
    for i in 0..64 {
        let bb: u64 = 1 << i;
        write!(white, "{},", nowe_one(bb) | noea_one(bb)).unwrap();
        write!(black, "{},", sowe_one(bb) | soea_one(bb)).unwrap();
    }
    return format!("[[{}],[{}]]", white, black).parse().unwrap();
}

#[proc_macro]
pub fn make_knight_attack_table(_item: TokenStream) -> TokenStream {
    assert!(_item.is_empty());
    let mut table = String::new();
    for i in 0..64 {
        let bb: u64 = 1 << i;
        let h1: u64 = ((bb >> 1) & 0x7f7f7f7f7f7f7f7f) | ((bb << 1) & 0xfefefefefefefefe);
        let h2: u64 = ((bb >> 2) & 0x3f3f3f3f3f3f3f3f) | ((bb << 2) & 0xfcfcfcfcfcfcfcfc);
        write!(table, "{},", (h1 << 16) | (h1 >> 16) | (h2 << 8) | (h2 >> 8)).unwrap();
    }
    return format!("[{}]", table).parse().unwrap();
}

#[proc_macro]
pub fn make_king_attack_table(_item: TokenStream) -> TokenStream {
    assert!(_item.is_empty());
    let mut table = String::new();
    for i in 0..64 {
        let mut bb: u64 = 1 << i;
        let mut attacks: u64 = east_one(bb) | west_one(bb);
        bb |= attacks;
        attacks |= nort_one(bb) | sout_one(bb);
        write!(table, "{},", attacks).unwrap();
    }
    return format!("[{}]", table).parse().unwrap();
}

fn rank_mask(sq: i32) -> u64 {
	return RANK1 << ((sq as u64) & 56); 
}

/*fn file_mask(sq: i32) -> u64 {
	return A_FILE << ((sq as u64) & 7);
}*/
fn diagonal_mask(sq: i32) -> u64 {
	let diag: i32 = 8 * (sq & 7) - (sq & 56);
	let nort: i32 = -diag & (diag >> 31);
	let sout: i32 = diag & (-diag >> 31);
	return (DIA_A1_H8 >> sout) << nort;
}
fn antidiag_mask(sq: i32) -> u64 {
	let diag: i32 = 56 - 8 * (sq & 7) - (sq & 56);
	let nort: i32 = -diag & (diag >> 31);
	let sout: i32 = diag & (-diag >> 31);
	return (DIA_H1_A8 >> sout) << nort;
}

#[proc_macro]
pub fn make_rank_mask_ex_table(_item: TokenStream) -> TokenStream {
    assert!(_item.is_empty());
    let mut table = String::new();
    for i in 0..64 {
        let bb: u64 = 1 << i;
        write!(table, "{},", rank_mask(i) ^ bb).unwrap();
    }
    return format!("[{}]", table).parse().unwrap();
}

#[proc_macro]
pub fn make_diagonal_mask_ex_table(_item: TokenStream) -> TokenStream {
    assert!(_item.is_empty());
    let mut table = String::new();
    for i in 0..64 {
        let bb: u64 = 1 << i;
        write!(table, "{},", diagonal_mask(i) ^ bb).unwrap();
    }
    return format!("[{}]", table).parse().unwrap();
}

#[proc_macro]
pub fn make_antidiag_mask_ex_table(_item: TokenStream) -> TokenStream {
    assert!(_item.is_empty());
    let mut table = String::new();
    for i in 0..64 {
        let bb: u64 = 1 << i;
        write!(table, "{},", antidiag_mask(i) ^ bb).unwrap();
    }
    return format!("[{}]", table).parse().unwrap();
}

fn east_attacks(mut rooks: u64, mut empty: u64) -> u64 {
	empty = empty & NOT_A_FILE; // make A-File all occupied, to consider H-A-wraps after shift
	rooks |= empty & (rooks << 1); // 1. fill
	rooks |= empty & (rooks << 1); // 2. fill
	rooks |= empty & (rooks << 1); // 3. fill
	rooks |= empty & (rooks << 1); // 4. fill
	rooks |= empty & (rooks << 1); // 5. fill
	rooks |= empty & (rooks << 1); // 6. fill
	return NOT_A_FILE & (rooks << 1);
}
fn west_attacks(mut rooks: u64, mut empty: u64) -> u64 {
	empty = empty & NOT_H_FILE;
	rooks |= empty & (rooks >> 1);
	rooks |= empty & (rooks >> 1);
	rooks |= empty & (rooks >> 1);
	rooks |= empty & (rooks >> 1);
	rooks |= empty & (rooks >> 1);
	rooks |= empty & (rooks >> 1);
	return NOT_H_FILE & (rooks >> 1);
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
fn nort_occluded(mut rooks: u64, mut empty: u64) -> u64 {
	rooks |= empty & (rooks << 8);
	empty = empty & (empty << 8);
	rooks |= empty & (rooks << 16);
	empty = empty & (empty << 16);
	rooks |= empty & (rooks << 32);
	return rooks;
}

#[proc_macro]
pub fn make_kindergarten_fill_up_attacks_table(_item: TokenStream) -> TokenStream {
    assert!(_item.is_empty());
    let mut table = String::new();
    for sq in 0..8 as usize {
        write!(table, "[").unwrap();
        for six_bit_occ in 0..64 as usize {
            let rooks: u64 = 1 << sq;
            let empty: u64 = !((six_bit_occ as u64) << 1);
            let first_rank_attacks: u64 = east_attacks(rooks, empty) | west_attacks(rooks, empty);

            write!(table, "{},", nort_occluded(first_rank_attacks, UNIVERSE_SET)).unwrap();
        }
        write!(table, "],").unwrap();
    }
    return format!("[{}]", table).parse().unwrap();
}

#[proc_macro]
pub fn make_kindergarten_a_file_attacks_table(_item: TokenStream) -> TokenStream {
    assert!(_item.is_empty());
    let mut table = String::new();
    for sq in 0..8 as usize {
        write!(table, "[").unwrap();
        for six_bit_occ in 0..64 as usize {
            let rooks: u64 = 1u64 << (7 - sq);
            let empty: u64 = !((six_bit_occ as u64) << 1);
            let first_rank_attacks: u64 = east_attacks(rooks, empty) | west_attacks(rooks, empty);

            let a_file_attack = ((first_rank_attacks.wrapping_mul(DIA_A1_H8)) >> 7) & A_FILE;
            write!(table, "{},", a_file_attack).unwrap();
        }
        write!(table, "],").unwrap();
    }
    return format!("[{}]", table).parse().unwrap();
}

use rand_chacha::rand_core::SeedableRng;
use rand_core::RngCore;
use rand_chacha; // 0.3.0

#[proc_macro]
pub fn make_random_u64_table(_item: TokenStream) -> TokenStream {
    let s = _item.to_string();
    let items = s.split(",").collect::<Vec<&str>>();
    let size;
    let seed;
    if items.len() == 2 {
        size = items[0].chars().filter(|c| !c.is_whitespace()).collect::<String>().parse::<usize>().unwrap();
        seed = items[1].chars().filter(|c| !c.is_whitespace()).collect::<String>().parse::<u64>().unwrap();
    }
    else {
        panic!("expected arguments are (size: usize, seed: u64)");
    }
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
    let mut table = String::new();
    for _ in 0..size as usize {
        write!(table, "{},", rng.next_u64()).unwrap();
    }
    return format!("[{}]", table).parse().unwrap();
}
