use crate::bitboard::{BitBoard, Square};
pub use indexing::{Color, Piece};

mod indexing;
mod move_calculation;

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
    #[must_use]
    pub fn empty() -> Board {
        Board {
            boards: [BitBoard::EMPTY; 8],
        }
    }
    #[inline(always)]
    #[must_use]
    pub fn get_color(&self, color: Color) -> BitBoard {
        self.boards[color as usize]
    }
    #[inline(always)]
    #[must_use]
    pub fn get_occupance(&self) -> BitBoard {
        self.get_color(Color::White) | self.get_color(Color::Black)
    }
    #[inline(always)]
    #[must_use]
    pub fn get_empty(&self) -> BitBoard {
        !self.get_occupance()
    }
    #[inline(always)]
    #[must_use]
    pub fn get_piece(&self, piece: Piece) -> BitBoard {
        self.boards[piece as usize + 2]
    }
    #[inline(always)]
    #[must_use]
    pub fn get_color_piece(&self, color: Color, piece: Piece) -> BitBoard {
        self.get_color(color) & self.get_piece(piece)
    }
    #[inline(always)]
    #[must_use]
    pub fn get_piece_at(&self, sq: Square) -> Option<Piece> {
        let bb = BitBoard::from(sq);
        if !(bb & self.get_piece(Piece::Pawn)).is_empty() {
            Some(Piece::Pawn)
        } else if !(bb & self.get_piece(Piece::Bishop)).is_empty() {
            Some(Piece::Bishop)
        } else if !(bb & self.get_piece(Piece::Knight)).is_empty() {
            Some(Piece::Knight)
        } else if !(bb & self.get_piece(Piece::Rook)).is_empty() {
            Some(Piece::Rook)
        } else if !(bb & self.get_piece(Piece::Queen)).is_empty() {
            Some(Piece::Queen)
        } else if !(bb & self.get_piece(Piece::King)).is_empty() {
            Some(Piece::King)
        } else {
            None
        }
    }
    #[inline(always)]
    #[must_use]
    pub fn get_color_at(&self, sq: Square) -> Option<Color> {
        let bb = BitBoard::from(sq);
        if !(bb & self.get_color(Color::White)).is_empty() {
            Some(Color::White)
        } else if !(bb & self.get_color(Color::Black)).is_empty() {
            Some(Color::Black)
        } else {
            None
        }
    }
    #[inline(always)]
    #[must_use]
    pub fn get_attackers_to(&self, sq: Square) -> BitBoard {
        let occ = self.get_occupance();

        BitBoard::pawn_attacks(sq, Color::White) & self.get_color_piece(Color::Black, Piece::Pawn)
            | BitBoard::pawn_attacks(sq, Color::Black)
                & self.get_color_piece(Color::White, Piece::Pawn)
            | BitBoard::knight_attacks(sq) & self.get_piece(Piece::Knight)
            | BitBoard::king_attacks(sq) & self.get_piece(Piece::King)
            | BitBoard::bishop_attacks(occ, sq) & self.get_bishop_sliders()
            | BitBoard::rook_attacks(occ, sq) & self.get_rook_sliders()
    }
    #[inline(always)]
    #[must_use]
    pub fn get_color_attackers_to(&self, sq: Square, color: Color) -> BitBoard {
        let occ = self.get_occupance();

        self.get_color(color)
            & (BitBoard::pawn_attacks(sq, !color) & self.get_piece(Piece::Pawn)
                | BitBoard::knight_attacks(sq) & self.get_piece(Piece::Knight)
                | BitBoard::king_attacks(sq) & self.get_piece(Piece::King)
                | BitBoard::bishop_attacks(occ, sq) & self.get_bishop_sliders()
                | BitBoard::rook_attacks(occ, sq) & self.get_rook_sliders())
    }
}

impl Board {
    #[inline(always)]
    #[must_use]
    pub fn get_bishop_sliders(&self) -> BitBoard {
        self.get_piece(Piece::Queen) | self.get_piece(Piece::Bishop)
    }
    #[inline(always)]
    #[must_use]
    pub fn get_color_bishop_sliders(&self, color: Color) -> BitBoard {
        self.get_color(color) & self.get_bishop_sliders()
    }
    #[inline(always)]
    #[must_use]
    pub fn get_rook_sliders(&self) -> BitBoard {
        self.get_piece(Piece::Queen) | self.get_piece(Piece::Rook)
    }
    #[inline(always)]
    #[must_use]
    pub fn get_color_rook_sliders(&self, color: Color) -> BitBoard {
        self.get_color(color) & self.get_rook_sliders()
    }
    // TODO: This function relies on a failable assumtion that the king exists.
    #[inline(always)]
    #[must_use]
    pub fn is_king_in_check(&self, color: Color) -> bool {
        !self
            .get_color_attackers_to(
                BitBoard::bit_scan_forward(self.get_color_piece(color, Piece::King))
                    .expect("king does not exist"),
                !color,
            )
            .is_empty()
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
