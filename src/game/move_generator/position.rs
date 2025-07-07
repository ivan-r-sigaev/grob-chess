// #[cfg(test)]
// mod testing;
// mod move_generation;

// use crate::board::*;
// use bitboard::*;
// use move_generation::*;

// use std::collections::HashMap;

// #[derive(Debug, Clone)]
// pub struct MoveGenerator {
//     board: Position,
//     move_list: MoveList,
//     unmove_list: Vec<UnmoveConcept>,
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum GameEnding {
//     Checkmate,
//     Stalemate,
// }

// impl MoveGenerator {
//     pub fn new(initial_board: Position) -> MoveGenerator {
//         return MoveGenerator { 
//             board: initial_board, 
//             move_list: MoveList::new(),
//             unmove_list: Vec::new(),
//         }
//     }
//     #[inline(always)]
//     fn next_node(&mut self) -> bool {
//         while let Some(next_move) = self.move_list.pop_move() {
//             let unmove = make_move(&mut self.board, next_move);
    
//             if !self.board.get_bitboard().is_king_in_check(!self.board.get_turn()) { 
//                 self.unmove_list.push(unmove);
//                 return true;
//             }
            
//             unmake_move(&mut self.board, unmove);
//         }

//         self.move_list.pop_group();
//         return false;
//     }
//     #[inline(always)]
//     pub fn inspect_child_nodes(&mut self) -> Option<GameEnding> {
//         self.move_list.generate_moves(&self.board);

//         if self.next_node() { return None; }

//         if self.board.get_bitboard().is_king_in_check(self.board.get_turn()) {
//             return Some(GameEnding::Checkmate);
//         }
//         else {
//             return Some(GameEnding::Stalemate);
//         }
//     }
//     #[inline(always)]
//     pub fn to_next_child_node(&mut self) -> bool {
//         unmake_move(&mut self.board, self.unmove_list.pop().unwrap());
//         return self.next_node();
//     }
//     #[inline(always)]
//     pub fn to_parent_node(&mut self) {
//         if self.unmove_list.is_empty() { panic!("no parent node exists") }
//         unmake_move(&mut self.board, self.unmove_list.pop().unwrap());
//         self.move_list.pop_group();
//     }
// }
pub use castling_rights::CastlingRights;
pub mod board;
use board::*;
use board::bitboard::*;

use crate::table_generation::*;
use std::error::Error;
use std::fmt::Display;
use std::mem::size_of;
use std::ops::Rem;

mod castling_rights;

#[inline(always)]
fn get_square_zobrist(color: Color, piece: Piece, sq: Square) -> u64 {
    const PIECE_AT_SQUARE_RANDOMS: [u64; 768] = make_random_u64_table::<768>();
    return PIECE_AT_SQUARE_RANDOMS[(piece as usize) * (color as usize) * (sq as usize)];
}

#[inline(always)]
fn get_turn_zobrist(turn: Color) -> u64 {
    // Size 1 is just a workaround to avoid making a different macro.
    const COLOR_RANDOMS: [u64; 1] = make_random_u64_table::<1>();  
    return if turn == Color::White { COLOR_RANDOMS[0] } else { 0 };
}

#[inline(always)]
fn get_en_passant_zobrist(en_passant: Option<File>) -> u64 {
    const EN_PASSANT_RANDOMS: [u64; 8] = make_random_u64_table::<8>();
    if let Some(file) = en_passant {
        return EN_PASSANT_RANDOMS[file as usize];
    }
    else { return 0; }
}

#[inline(always)]
fn get_castling_zobrist(castling_rights: CastlingRights) -> u64 {
    const CASTLING_RANDOMS: [u64; 16] = make_random_u64_table::<16>();
    return CASTLING_RANDOMS[castling_rights.bits() as usize];
}

// #[derive(Debug, Clone, Copy)]
// pub struct PieceScore {
//     pub pawn_count: u32,
//     pub bishop_count: u32,
//     pub knight_count: u32,
//     pub rook_count: u32,
//     pub queen_count: u32
// }

#[derive(Debug, Clone, Copy)]
pub struct Position {
    board: Board,
    turn: Color,
    castling_rights: CastlingRights,
    en_passant: Option<File>,
    halfmove_clock: u32,
    zobrist_hash: u64,
    //piece_scores: [PieceScore; 2],
}

impl PartialEq for Position {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        return self.zobrist_hash == other.zobrist_hash  // Compare hash to maybe get a faster comparison on average.
        && self.board == other.board 
        && self.turn == other.turn
        && self.castling_rights == other.castling_rights
        && self.en_passant == other.en_passant;
    }
}

impl Eq for Position {}

#[derive(Debug, Clone)]
pub enum ParseFenError {
    BadFenSize,
    BadRowCount,
    BadRowSize,
    BadCastlingRights,
    BadEnPassant,
    BadHalfmoveClock,
    BadFullmoveClock,
    UnknownCharacter,
}

impl Display for ParseFenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{:?}", self);
    }
}

impl Error for ParseFenError {}

impl Position {
    pub fn try_from_fen(fen: &str) -> Result<Position, ParseFenError> {
        let fen_parts: Vec<&str> = fen.split_whitespace().collect();
        if fen_parts.len() != 6 { return Err(ParseFenError::BadFenSize); }

        let position_parts: Vec<&str> = fen_parts[0].split("/").collect();
        if position_parts.len() != 8 { return Err(ParseFenError::BadRowCount); }

        let mut bitboard = Board::empty();
        let mut zobrist_hash = 0;
        let mut sq: Square = Square::A1;
        for y in (0..8).rev() {
            let mut x = 0;
            for ch in position_parts[y].chars().into_iter() {
                if ch >= '1' && ch <= '8' {
                    let increment = u8::try_from(ch).unwrap() - u8::try_from('1').unwrap();
                    x += increment;
                    sq = sq.shifted(increment as i8);
                }
                else if ch.is_ascii_alphabetic() {
                    let lower = ch.to_ascii_lowercase();
                    let color = if ch.is_ascii_lowercase() {Color::Black} else {Color::White};

                    let piece = if lower == 'k' { Piece::King }
                    else if lower == 'q' { Piece::Queen }
                    else if lower == 'r' { Piece::Rook }
                    else if lower == 'b' { Piece::Bishop }
                    else if lower == 'n' { Piece::Knight }
                    else if lower == 'p' { Piece::Pawn }
                    else { return Err(ParseFenError::UnknownCharacter); };
                    
                    bitboard.mask_or(color, piece, BitBoard::from(sq));
                    zobrist_hash ^= get_square_zobrist(color, piece, sq);
                }
                else { return Err(ParseFenError::UnknownCharacter); }
                sq = sq.shifted(1);
                x += 1;
                if x > 8 { return Err(ParseFenError::BadFenSize); }
            }
            if x < 8 { return Err(ParseFenError::BadFenSize); }
        }

        let turn = if fen_parts[1] == "w" { Color::White }
        else if fen_parts[1] == "b" { Color::Black }
        else { return Err(ParseFenError::UnknownCharacter); };

        zobrist_hash ^= get_turn_zobrist(turn);

        let mut castling_rights = CastlingRights::empty();
        if fen_parts[2] !=  "-" {
            let mut remaining_len = fen_parts[2].len();
            if fen_parts[2].find("K").is_some() {
                remaining_len -= 1;
                castling_rights |= CastlingRights::WHITE_KING;
            }
            if fen_parts[2].find("Q").is_some() {
                remaining_len -= 1;
                castling_rights |= CastlingRights::WHITE_QUEEN;
            }
            if fen_parts[2].find("k").is_some() {
                remaining_len -= 1;
                castling_rights |= CastlingRights::BLACK_KING;
            }
            if fen_parts[2].find("q").is_some() {
                remaining_len -= 1;
                castling_rights |= CastlingRights::BLACK_QUEEN;
            }
            if remaining_len != 0 { return Err(ParseFenError::BadCastlingRights); }
        }
        zobrist_hash ^= get_castling_zobrist(castling_rights);

        let en_passant;
        if fen_parts[3] != "-" {
            if fen_parts[3].chars().count() != 2 { return Err(ParseFenError::BadEnPassant); }
            let col = fen_parts[3].chars().nth(0).unwrap();
            let row = fen_parts[3].chars().nth(1).unwrap();
            if row != '6' || row != '3' { return Err(ParseFenError::BadEnPassant); }

            if col == 'a' {en_passant = Some(File::A)}
            else if col == 'b' {en_passant = Some(File::B)}
            else if col == 'c' {en_passant = Some(File::C)}
            else if col == 'd' {en_passant = Some(File::D)}
            else if col == 'e' {en_passant = Some(File::E)}
            else if col == 'f' {en_passant = Some(File::F)}
            else if col == 'g' {en_passant = Some(File::G)}
            else if col == 'h' {en_passant = Some(File::H)}
            else { return Err(ParseFenError::BadEnPassant); }
        }
        else {
            en_passant = None;
        }
        zobrist_hash ^= get_en_passant_zobrist(en_passant);

        let hm = fen_parts[4].parse::<u32>();
        if hm.is_err() { return Err(ParseFenError::BadHalfmoveClock); }
        let halfmove_clock = hm.unwrap();

        let fm = fen_parts[5].parse::<u32>();
        if fm.is_err() { return Err(ParseFenError::BadFullmoveClock); }

        // let piece_scores: [PieceScore; 2] = [
        //     board.get_piece_score(Color::try_from(0)),
        //     board.get_piece_score(Color::Black)
        // ];

        return Ok(Position { 
            board: bitboard, 
            turn, 
            castling_rights, 
            en_passant, 
            halfmove_clock, 
            zobrist_hash, 
            //piece_scores,
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PositionHash(u64);

impl Rem<usize> for PositionHash {
    type Output = usize;

    fn rem(self, rhs: usize) -> Self::Output {
        const MAX: u64 = if size_of::<u64>() > size_of::<usize>() { usize::MAX as u64 } else { u64::MAX };
        return (self.0 & MAX) as usize % rhs;
    }
}

impl Position {
    #[inline(always)]
    pub fn position_hash(&self) -> PositionHash {
        return PositionHash(self.zobrist_hash);
    }
    #[inline(always)]
    pub fn en_passant(&self) -> Option<File> {
        return self.en_passant;
    }
    #[inline(always)]
    pub fn castling_rights(&self) -> CastlingRights {
        return self.castling_rights;
    }
    #[inline(always)]
    pub fn board(&self) -> &Board {
        return &self.board;
    }
    #[inline(always)]
    pub fn turn(&self) -> Color {
        return self.turn;
    }
    #[inline(always)]
    pub fn halfmove_clock(&self) -> u32 {
        return self.halfmove_clock;
    }
}

impl Position {
    #[inline(always)]
    pub fn set_en_passant(&mut self, en_passant: Option<File>) {
        self.zobrist_hash ^= get_en_passant_zobrist(self.en_passant);
        self.zobrist_hash ^= get_en_passant_zobrist(en_passant);
        self.en_passant = en_passant;
    }
    #[inline(always)]
    pub fn set_castling_rights(&mut self, castling_rights: CastlingRights) {
        self.zobrist_hash ^= get_castling_zobrist(self.castling_rights);
        self.zobrist_hash ^= get_castling_zobrist(castling_rights);
        self.castling_rights = castling_rights;
    }
    #[inline(always)]
    pub fn add_color_piece(&mut self, color: Color, piece: Piece, sq: Square) {
        debug_assert!(!self.board.get_occupance().has_square(sq));
        self.zobrist_hash ^= get_square_zobrist(color, piece, sq);
        self.board.mask_or(color, piece, BitBoard::from(sq));
    }
    #[inline(always)]
    pub fn remove_color_piece(&mut self, color: Color, piece: Piece, sq: Square) {
        debug_assert!(self.board.get_piece(piece).has_square(sq) && self.board.get_color(color).has_square(sq));
        self.zobrist_hash ^= get_square_zobrist(color, piece, sq);
        self.board.mask_and(color, piece, !BitBoard::from(sq));
    }
    #[inline(always)]
    pub fn move_color_piece(&mut self, color: Color, piece: Piece, from: Square, to: Square) {
        debug_assert!(self.board.get_piece(piece).has_square(from) && self.board.get_color(color).has_square(from));
        debug_assert!(!self.board.get_occupance().has_square(to));
        self.zobrist_hash ^= get_square_zobrist(color, piece, from);
        self.zobrist_hash ^= get_square_zobrist(color, piece, to);
        self.board.mask_xor(color, piece, BitBoard::from(from) | BitBoard::from(to));
    }
    #[inline(always)]
    pub fn set_turn(&mut self, turn: Color) {
        self.zobrist_hash ^= get_turn_zobrist(self.turn);
        self.zobrist_hash ^= get_turn_zobrist(turn);
        self.turn = turn;
    }
    #[inline(always)]
    pub fn set_halfmove_clock(&mut self, halfmove_clock: u32) {
        self.halfmove_clock = halfmove_clock;
    }
}

impl Position {
    #[inline(always)]
    pub fn is_kingside_castling_prohibited(&self, color: Color) -> bool {
	    // TODO: remove crights when rook is taken instead of checking for it's existence
        let w_empty = BitBoard::from(Square::F1) | BitBoard::from(Square::G1);
        let b_empty = BitBoard::from(Square::F8) | BitBoard::from(Square::G8);
        return !self.castling_rights.contains(CastlingRights::kingside(self.turn))
            || (self.board.get_color_piece(color, Piece::Rook) & BitBoard::from(if color == Color::White { Square::H1 } else { Square::H8 })).none()
            || !(self.board.get_occupance() & if color == Color::White { w_empty } else { b_empty }).none()
            || !(self.board.get_color_attackers_to(if color == Color::White { Square::F1 } else { Square::F8 }, !color)).none()
            || !(self.board.get_color_attackers_to(if color == Color::White { Square::G1 } else { Square::G8 }, !color)).none();
    }
    #[inline(always)]
    pub fn is_queenside_castling_prohibited(&self, color: Color) -> bool {
	    // TODO: remove crights when rook is taken instead of checking for it's existence
        let w_empty = BitBoard::from(Square::B1) | BitBoard::from(Square::C1) | BitBoard::from(Square::D1);
        let b_empty = BitBoard::from(Square::B8) | BitBoard::from(Square::C8) | BitBoard::from(Square::D8);
        return !self.castling_rights.contains(CastlingRights::queenside(self.turn))
            || (self.board.get_color_piece(color, Piece::Rook) & BitBoard::from(if color == Color::White { Square::A1 } else { Square::A8 })).none()
            || !(self.board.get_occupance() & if color == Color::White { w_empty } else { b_empty }).none()
            || !(self.board.get_color_attackers_to(if color == Color::White { Square::C1 } else { Square::C8 }, !color)).none()
            || !(self.board.get_color_attackers_to(if color == Color::White { Square::D1 } else { Square::D8 }, !color)).none();
    }
}
