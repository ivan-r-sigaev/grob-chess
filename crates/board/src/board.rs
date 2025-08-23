use std::fmt;
use strum::IntoEnumIterator;

use crate::{BitBoard, Color, File, Piece, Rank, Square};

/// Pieces that are placed on the board.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
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

    /// Returns an empty board.
    #[inline(always)]
    #[must_use]
    pub fn empty() -> Self {
        Self {
            boards: [BitBoard::EMPTY; 8],
        }
    }

    /// Returns the bitboard with pieces of this color.
    #[inline(always)]
    #[must_use]
    pub fn get_color(&self, color: Color) -> BitBoard {
        self.boards[color as usize]
    }

    /// Returns the bitboard with all pieces.
    #[inline(always)]
    #[must_use]
    pub fn get_occupance(&self) -> BitBoard {
        self.get_color(Color::White) | self.get_color(Color::Black)
    }

    /// Returns the bitboard with the unoccupied squares.
    #[inline(always)]
    #[must_use]
    pub fn get_empty(&self) -> BitBoard {
        !self.get_occupance()
    }

    /// Returns the bitboard with pieces of the given type.
    #[inline(always)]
    #[must_use]
    pub fn get_piece(&self, piece: Piece) -> BitBoard {
        self.boards[piece as usize + 2]
    }

    /// Returns the bitboard with pieces that all share
    /// the given color and a given type.
    #[inline(always)]
    #[must_use]
    pub fn get_color_piece(&self, color: Color, piece: Piece) -> BitBoard {
        self.get_color(color) & self.get_piece(piece)
    }

    /// Returns the type of the piece on the given square if the piece is present.
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

    /// Returns the color of the piece on the given square if the piece is persent.
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

    /// Returns the bitboard with the pieces that attack (put pressure on) the given square.
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

    /// Same as [`Board::get_attackers_to`], but only returns pieces of the given color.
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

    /// Returns whether a king can step on a given square.
    pub fn can_king_move_to(&self, sq: Square, color: Color) -> bool {
        !self.get_occupance().has_square(sq) && self.get_color_attackers_to(sq, !color).is_empty()
    }

    /// Returns the bitboard with queens and bishops.
    #[inline(always)]
    #[must_use]
    pub fn get_bishop_sliders(&self) -> BitBoard {
        self.get_piece(Piece::Queen) | self.get_piece(Piece::Bishop)
    }

    /// Same as [`Board::get_bishop_sliders`], but only returns pieces of the given color.
    #[inline(always)]
    #[must_use]
    pub fn get_color_bishop_sliders(&self, color: Color) -> BitBoard {
        self.get_color(color) & self.get_bishop_sliders()
    }

    /// Returns the bitboard with queens and rooks.
    #[inline(always)]
    #[must_use]
    pub fn get_rook_sliders(&self) -> BitBoard {
        self.get_piece(Piece::Queen) | self.get_piece(Piece::Rook)
    }

    /// Same as [`Board::get_rook_sliders`], but only returns pieces of the given color.
    #[inline(always)]
    #[must_use]
    pub fn get_color_rook_sliders(&self, color: Color) -> BitBoard {
        self.get_color(color) & self.get_rook_sliders()
    }

    /// Returns the square of the king.
    ///
    /// # Panics
    /// Panics if the board does not have a king of this color.
    pub fn get_king(&self, color: Color) -> Square {
        BitBoard::bit_scan_forward(self.get_color_piece(color, Piece::King))
            .expect("king does not exist")
    }

    /// Returns the pieces declaring check to the king of this color.
    ///
    /// # Panics
    /// Panics if the board does not have a king of this color.
    pub fn get_king_checkers(&self, color: Color) -> BitBoard {
        self.get_color_attackers_to(self.get_king(color), !color)
    }

    /// Returns `true` if the king of the given color is currently in check.
    ///
    /// # Panics
    /// Panics if the board does not have a king of this color.
    #[inline(always)]
    #[must_use]
    pub fn is_king_in_check(&self, color: Color) -> bool {
        !self.get_king_checkers(color).is_empty()
    }
}

impl Board {
    /// Places (or replaces) pieces of the given color on the squares
    /// specified by the mask with the given piece type.
    ///
    /// # Preconditions
    ///
    /// The user of the function is responsible for not trying to overwrite the squares that contain
    /// the opposite color, which will result in doubly colored pieces.
    #[inline(always)]
    pub fn mask_or(&mut self, color: Color, piece: Piece, mask: BitBoard) {
        self.boards[piece as usize + 2] |= mask;
        self.boards[color as usize] |= mask;
    }

    /// Removes all pieces of the given color and type
    /// on the squares NOT specified by the mask.
    ///
    /// # Preconditions
    ///
    /// The user of the function is responsible for not trying to remove the pieces of a different color
    /// than specified by the mask, which will result in colored squares without a piece type.
    #[inline(always)]
    pub fn mask_and(&mut self, color: Color, piece: Piece, mask: BitBoard) {
        self.boards[piece as usize + 2] &= mask;
        self.boards[color as usize] &= mask;
    }

    /// Toggles all the pieces of the given color and type
    /// on the squares specified by the mask.
    ///
    /// # Preconditions
    ///
    /// The user of the function is responsible for not trying to toggle the pieces of a different color
    /// or a different piece type than specified by the mask, which will result in one of the following:
    /// - multicolored pieces
    /// - multityped pieces
    /// - uncolored pieces
    /// - colored squares without a piece type
    /// - severe headaches and vomiting
    /// - immediate heat death of the universe
    /// - \[REDACTED\]
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
                white: {{\n{}}},\n\
                black: {{\n{}}},\n\
                pawn: {{\n{}}},\n\
                bishop: {{\n{}}},\n\
                knight: {{\n{}}},\n\
                rook: {{\n{}}},\n\
                queen: {{\n{}}},\n\
                king: {{\n{}}},\n\
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

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut drawing = String::new();
        for rank in Rank::iter().rev() {
            drawing += "  ";
            for file in File::iter() {
                let sq = Square::new(rank, file);
                let piece = self.get_piece_at(sq);
                let color = self.get_color_at(sq);
                if let Some((color, piece)) = color.zip(piece) {
                    drawing += &format!("{color}{piece}");
                } else {
                    drawing += "__";
                }
                drawing += " ";
            }
            drawing += "\n"
        }
        write!(f, "Chess board {{\n{drawing}}}")
    }
}
