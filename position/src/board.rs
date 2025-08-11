pub use crate::bitboard::BitBoard;
pub use crate::pieces::{Color, Piece, Promotion};
pub use crate::square::{File, NegDiag, PosDiag, Rank, Square};

use std::fmt;
use strum::IntoEnumIterator;

/// Current state of all the pieces on the chess board.
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
    /// Constructs an empty board.
    ///
    /// # Returns
    /// `Self` - an empty board.
    #[inline(always)]
    #[must_use]
    pub fn empty() -> Self {
        Self {
            boards: [BitBoard::EMPTY; 8],
        }
    }

    /// Returns the bitboard with all the pieces of the given color.
    ///
    /// # Arguments
    /// * `color` - the given color
    ///
    /// # Returns
    /// * `BitBoard` - the bitboard with all the pieces of the given color
    #[inline(always)]
    #[must_use]
    pub fn get_color(&self, color: Color) -> BitBoard {
        self.boards[color as usize]
    }

    /// Returns the bitboard with all the pieces currently present on the board.
    ///
    /// # Returns
    /// * `BitBoard` - the bitboard with all the pieces currently present on the board
    #[inline(always)]
    #[must_use]
    pub fn get_occupance(&self) -> BitBoard {
        self.get_color(Color::White) | self.get_color(Color::Black)
    }

    /// Returns the bitboard with all the unoccupied squares.
    ///
    /// # Returns
    /// * `BitBoard` - the bitboard with all the unoccupied squares
    #[inline(always)]
    #[must_use]
    pub fn get_empty(&self) -> BitBoard {
        !self.get_occupance()
    }

    /// Returns the bitboard with all the pieces of the given piece type.
    ///
    /// # Arguments
    /// * `piece` - the given piece type
    ///
    /// # Returns
    /// * `BitBoard` - the bitboard with all the pieces of the given piece type
    #[inline(always)]
    #[must_use]
    pub fn get_piece(&self, piece: Piece) -> BitBoard {
        self.boards[piece as usize + 2]
    }

    /// Returns the bitboard with all the pieces that are BOTH the of given piece type and of the given color.
    ///
    /// # Arguments
    /// * `color` - the given color
    /// * `piece` - the given piece type
    ///
    /// # Returns
    /// * `BitBoard` - the bitboard with all the pieces that are BOTH the of given piece type and of the given color
    #[inline(always)]
    #[must_use]
    pub fn get_color_piece(&self, color: Color, piece: Piece) -> BitBoard {
        self.get_color(color) & self.get_piece(piece)
    }

    /// Returns the piece type placed on the given square (or `None` if the square is empty).
    ///
    /// # Arguments
    /// * `sq` - the given square
    ///
    /// # Returns
    /// `Option<Piece>`:
    /// - `Some(piece: Piece)` - the piece type placed on the given square
    /// - `None` - if the square is empty
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

    /// Returns the color of the piece placed on the given square (or `None` if the square is empty).
    ///
    /// # Arguments
    /// * `sq` - the given square
    ///
    /// # Returns
    /// `Option<Color>`:
    /// - `Some(piece: Color)` - the color of the piece placed on the given square
    /// - `None` - if the square is empty
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

    /// Returns the bitboard with all the pieces that attack (put pressure on) the given square.
    ///
    /// # Arguments
    /// * `sq` - the given square
    ///
    /// # Returns
    /// `BitBoard` - the bitboard with all the pieces that attack (put pressure on) the given square
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

    /// Returns the bitboard with all the pieces of a given color that attack (put pressure on) the given square.
    ///
    /// # Arguments
    /// * `sq` - the given square
    /// * `color` - the given color
    ///
    /// # Returns
    /// `BitBoard` - the bitboard with all the pieces of a given color that attack (put pressure on) the given square
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
    /// Returns the bitboard with all the queens and bishops.
    ///
    /// # Returns
    /// `BitBoard` - the bitboard with all the queens and bishops
    #[inline(always)]
    #[must_use]
    pub fn get_bishop_sliders(&self) -> BitBoard {
        self.get_piece(Piece::Queen) | self.get_piece(Piece::Bishop)
    }

    /// Returns the bitboard with all the queens and bishops of the given color.
    ///
    /// # Arguments
    /// * `color` - the given color
    ///
    /// # Returns
    /// `BitBoard` - the bitboard with all the queens and bishops of the given color
    #[inline(always)]
    #[must_use]
    pub fn get_color_bishop_sliders(&self, color: Color) -> BitBoard {
        self.get_color(color) & self.get_bishop_sliders()
    }

    /// Returns the bitboard with all the queens and rooks.
    ///
    /// # Returns
    /// `BitBoard` - the bitboard with all the queens and rooks
    #[inline(always)]
    #[must_use]
    pub fn get_rook_sliders(&self) -> BitBoard {
        self.get_piece(Piece::Queen) | self.get_piece(Piece::Rook)
    }

    /// Returns the bitboard with all the rooks and bishops of the given color.
    ///
    /// # Arguments
    /// * `color` - the given color
    ///
    /// # Returns
    /// `BitBoard` - the bitboard with all the queens and rooks of the given color
    #[inline(always)]
    #[must_use]
    pub fn get_color_rook_sliders(&self, color: Color) -> BitBoard {
        self.get_color(color) & self.get_rook_sliders()
    }

    // TODO: This function relies on a failable assumtion that the king exists.
    /// Returns whether the king of the given color is currently in check.
    ///
    /// # Arguments
    /// * `color` - the given color
    ///
    /// # Returns
    /// `bool` - whether the king of the given color is currently in check
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
    /// Places (or replaces) pieces of the given color on the squares specified by the mask with the given piece type.
    ///
    /// # Preconditions
    ///
    /// The user of the function is responsible for not trying to overwrite the squares that contain
    /// the opposite color, which will result in doubly colored pieces.
    ///
    /// # Arguments
    /// * `color` - the color of pieces
    /// * `piece` - the type of the pieces to place
    /// * `mask` - the mask where to place (or replace) the pieces
    #[inline(always)]
    pub(crate) fn mask_or(&mut self, color: Color, piece: Piece, mask: BitBoard) {
        self.boards[piece as usize + 2] |= mask;
        self.boards[color as usize] |= mask;
    }

    /// Removes all pieces of the given color and type on the squares NOT specified by the mask.
    ///
    /// # Preconditions
    ///
    /// The user of the function is responsible for not trying to remove the pieces of a different color
    /// than specified by the mask, which will result in colored squares without a piece type.
    ///
    /// # Arguments
    /// * `color` - the color of the pieces
    /// * `piece` - the type of the pieces
    /// * `mask` - the mask specifying what pieces to keep
    #[inline(always)]
    pub(crate) fn mask_and(&mut self, color: Color, piece: Piece, mask: BitBoard) {
        self.boards[piece as usize + 2] &= mask;
        self.boards[color as usize] &= mask;
    }

    /// Toggles all the pieces of the given color and type on the squares specified by the mask.
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
    ///
    /// # Arguments
    /// * `color` - the color of the pieces
    /// * `piece` - the type of the pieces
    /// * `mask` - the mask specifying what pieces to toggle
    #[inline(always)]
    pub(crate) fn mask_xor(&mut self, color: Color, piece: Piece, mask: BitBoard) {
        self.boards[piece as usize + 2] ^= mask;
        self.boards[color as usize] ^= mask;
    }
}

impl std::fmt::Debug for Board {
    /// Formats board for debug purposes.
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
