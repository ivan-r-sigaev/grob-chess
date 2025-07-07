pub use indexing::{Color, Piece};
pub mod bitboard;
use bitboard::*;

mod indexing;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Board {
    /*
    White  = 0,
    Black  = 1,
    Pawn   = 2,
    Bishop = 3,
    Knight = 4,
    Rook   = 5,
    Queen  = 6,
    King   = 7
    */
    boards: [BitBoard; 8],
}

impl Board {
    // TODO: it may be better to add a constructor from FEN.
    #[inline(always)]
    pub fn empty() -> Board {
        return Board {
            boards: [BitBoard::EMPTY; 8],
        };
    }
    #[inline(always)]
    pub fn get_color(&self, color: Color) -> BitBoard {
        return self.boards[color as usize];
    }
    #[inline(always)]
    pub fn get_occupance(&self) -> BitBoard {
        return self.get_color(Color::White) | self.get_color(Color::Black);
    }
    #[inline(always)]
    pub fn get_empty(&self) -> BitBoard {
        return !self.get_occupance();
    }
    #[inline(always)]
    pub fn get_piece(&self, piece: Piece) -> BitBoard {
        return self.boards[piece as usize + 2];
    }
    #[inline(always)]
    pub fn get_color_piece(&self, color: Color, piece: Piece) -> BitBoard {
        return self.get_color(color) & self.get_piece(piece);
    }
    #[inline(always)]
    pub fn get_piece_at(&self, sq: Square) -> Option<Piece> {
        let bb = BitBoard::from(sq);
        return if !(bb & self.get_piece(Piece::Pawn)).none() {
            Some(Piece::Pawn)
        } else if !(bb & self.get_piece(Piece::Bishop)).none() {
            Some(Piece::Bishop)
        } else if !(bb & self.get_piece(Piece::Knight)).none() {
            Some(Piece::Knight)
        } else if !(bb & self.get_piece(Piece::Rook)).none() {
            Some(Piece::Rook)
        } else if !(bb & self.get_piece(Piece::Queen)).none() {
            Some(Piece::Queen)
        } else if !(bb & self.get_piece(Piece::King)).none() {
            Some(Piece::King)
        } else {
            None
        };
    }
    #[inline(always)]
    pub fn get_color_at(&self, sq: Square) -> Option<Color> {
        let bb = BitBoard::from(sq);
        if !(bb & self.get_color(Color::White)).none() {
            Some(Color::White)
        } else if !(bb & self.get_color(Color::Black)).none() {
            Some(Color::Black)
        } else {
            None
        }
    }
    #[inline(always)]
    pub fn get_attackers_to(&self, sq: Square) -> BitBoard {
        let occ = self.get_occupance();

        return BitBoard::pawn_attacks(sq, Color::White)
            & self.get_color_piece(Color::Black, Piece::Pawn)
            | BitBoard::pawn_attacks(sq, Color::Black)
                & self.get_color_piece(Color::White, Piece::Pawn)
            | BitBoard::knight_attacks(sq) & self.get_piece(Piece::Knight)
            | BitBoard::king_attacks(sq) & self.get_piece(Piece::King)
            | BitBoard::bishop_attacks(occ, sq) & self.get_bishop_sliders()
            | BitBoard::rook_attacks(occ, sq) & self.get_rook_sliders();
    }
    #[inline(always)]
    pub fn get_color_attackers_to(&self, sq: Square, color: Color) -> BitBoard {
        let occ = self.get_occupance();

        return self.get_color(color)
            & (BitBoard::pawn_attacks(sq, !color) & self.get_piece(Piece::Pawn)
                | BitBoard::knight_attacks(sq) & self.get_piece(Piece::Knight)
                | BitBoard::king_attacks(sq) & self.get_piece(Piece::King)
                | BitBoard::bishop_attacks(occ, sq) & self.get_bishop_sliders()
                | BitBoard::rook_attacks(occ, sq) & self.get_rook_sliders());
    }
    // #[inline(always)]
    // pub fn get_piece_score(&self, color: Color) -> PieceScore {
    //     return PieceScore {
    //         pawn_count: self.get_color_piece(color, Piece::Pawn).count_ones(),
    //         bishop_count: self.get_color_piece(color, Piece::Bishop).count_ones(),
    //         knight_count: self.get_color_piece(color, Piece::Knight).count_ones(),
    //         rook_count: self.get_color_piece(color, Piece::Rook).count_ones(),
    //         queen_count: self.get_color_piece(color, Piece::Queen).count_ones()
    //     }
    // }
}

impl Board {
    #[inline(always)]
    pub fn get_bishop_sliders(&self) -> BitBoard {
        return self.get_piece(Piece::Queen) | self.get_piece(Piece::Bishop);
    }
    #[inline(always)]
    pub fn get_color_bishop_sliders(&self, color: Color) -> BitBoard {
        return self.get_color(color) & self.get_bishop_sliders();
    }
    #[inline(always)]
    pub fn get_rook_sliders(&self) -> BitBoard {
        return self.get_piece(Piece::Queen) | self.get_piece(Piece::Rook);
    }
    #[inline(always)]
    pub fn get_color_rook_sliders(&self, color: Color) -> BitBoard {
        return self.get_color(color) & self.get_rook_sliders();
    }
    // TODO: This function relies on a failable assumtion that the king exists.
    #[inline(always)]
    pub fn is_king_in_check(&self, color: Color) -> bool {
        return !self
            .get_color_attackers_to(
                BitBoard::bit_scan_forward(self.get_color_piece(color, Piece::King))
                    .expect("king does not exist"),
                !color,
            )
            .none();
    }
}

impl Board {
    #[inline(always)]
    pub fn mask_or(&mut self, color: Color, piece: Piece, mask: BitBoard) {
        self.boards[piece as usize + 2] |= mask;
        self.boards[color as usize] |= mask;
    }
    #[inline(always)]
    pub fn mask_and(&mut self, color: Color, piece: Piece, mask: BitBoard) {
        self.boards[piece as usize + 2] &= mask;
        self.boards[color as usize] &= mask;
    }
    #[inline(always)]
    pub fn mask_xor(&mut self, color: Color, piece: Piece, mask: BitBoard) {
        self.boards[piece as usize + 2] ^= mask;
        self.boards[color as usize] ^= mask;
    }
}

impl std::fmt::Debug for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{{\
                white: {{\n{:?}}},\n\
                black: {{\n{:?}}},\n\
                pawn: {{\n{:?}}},\n\
                bishop: {{\n{:?}}},\n\
                knight: {{\n{:?}}},\n\
                rook: {{\n{:?}}},\n\
                queen: {{\n{:?}}},\n\
                king: {{\n{:?}}},\n\
            }}",
            self.get_color(Color::White),
            self.get_color(Color::Black),
            self.get_piece(Piece::Pawn),
            self.get_piece(Piece::Bishop),
            self.get_piece(Piece::Knight),
            self.get_piece(Piece::Rook),
            self.get_piece(Piece::Queen),
            self.get_piece(Piece::King),
        )
    }
}

// pub fn format_position(position: &Board) -> String {
//     let mut formatted = String::new();
//     for y in (0..8).rev() {
//         for x in 0..8 {
//             let sq = Square::try_from(y * 8 + x).unwrap();
//             formatted += match position.get_bitboard().get_piece_at(sq) {
//                 Some(piece) => {
//                     let is_white = (1u64 << sq as u8) & position.get_bitboard().get_color(Color::White) != 0;
//                     match piece {
//                         Piece::Pawn => if is_white { "P" } else { "p" },
//                         Piece::Bishop => if is_white { "B" } else { "b" },
//                         Piece::Knight => if is_white { "N" } else { "n" },
//                         Piece::Rook => if is_white { "R" } else { "r" },
//                         Piece::Queen => if is_white { "Q" } else { "q" },
//                         Piece::King => if is_white { "K" } else { "k" },
//                     }
//                 }
//                 None => "_",
//             };
//             formatted += " ";
//         }
//         formatted += "\n";
//     }
//     return formatted;
// }
