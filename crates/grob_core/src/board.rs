use std::fmt;
use strum::{EnumCount, IntoEnumIterator};

use crate::{
    bitboard::BitBoard,
    pieces::{piece_to_fen, Color, Piece},
    square::{File, Rank, Square},
};

/// Stores all the occupancy masks ([`BitBoard`]s) in chess position.
///
/// Effectively represents which pieces are placed on the board.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Board {
    /// Occupancies by color.
    colors: [BitBoard; Color::COUNT],
    /// Occupancies by piece kind.
    pieces: [BitBoard; Piece::COUNT],
}

impl Board {
    /// Constructs a new empty [`Board`].
    #[inline(always)]
    #[must_use]
    pub fn empty() -> Self {
        Self {
            colors: [BitBoard::EMPTY; Color::COUNT],
            pieces: [BitBoard::EMPTY; Piece::COUNT],
        }
    }
    /// Returns all pieces of specified color.
    #[inline(always)]
    #[must_use]
    pub fn get_color(&self, color: Color) -> BitBoard {
        self.colors[color as usize]
    }
    /// Returns all pieces of specified kind.
    #[inline(always)]
    #[must_use]
    pub fn get_piece(&self, piece: Piece) -> BitBoard {
        self.pieces[piece as usize]
    }
    /// Returns all pieces of the specified color that have the specified kind.
    #[inline(always)]
    #[must_use]
    pub fn get_color_piece(&self, color: Color, piece: Piece) -> BitBoard {
        self.get_color(color) & self.get_piece(piece)
    }
    /// Returns all the pieces on the board.
    #[inline(always)]
    #[must_use]
    pub fn get_occupied(&self) -> BitBoard {
        self.get_color(Color::White) | self.get_color(Color::Black)
    }
    /// Returns all the empty squares.
    #[inline(always)]
    #[must_use]
    pub fn get_empty(&self) -> BitBoard {
        !self.get_occupied()
    }
    /// Returns the kind of the piece at the specified square.
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
    /// Returns the color of the piece at the specified square.
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
    /// Returns the color and the kind of the piece at the specified square.
    pub fn get_color_piece_at(&self, sq: Square) -> Option<(Color, Piece)> {
        let color = self.get_color_at(sq)?;
        let piece = self.get_piece_at(sq).unwrap();
        Some((color, piece))
    }
    /// Returns all piece attacking (putting pressure on) the specified square.
    #[inline(always)]
    #[must_use]
    pub fn get_attackers_to(&self, sq: Square) -> BitBoard {
        let occ = self.get_occupied();

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
        let occ = self.get_occupied();

        self.get_color(color)
            & (BitBoard::pawn_attacks(sq, !color) & self.get_piece(Piece::Pawn)
                | BitBoard::knight_attacks(sq) & self.get_piece(Piece::Knight)
                | BitBoard::king_attacks(sq) & self.get_piece(Piece::King)
                | BitBoard::bishop_attacks(occ, sq) & self.get_bishop_sliders()
                | BitBoard::rook_attacks(occ, sq) & self.get_rook_sliders())
    }
    /// Returns `true` if the specified square is empty and the king
    /// of specified color wouldn't be in check on this square.
    pub fn can_king_move_to(&self, sq: Square, color: Color) -> bool {
        !self.get_occupied().has_square(sq) && self.get_color_attackers_to(sq, !color).is_empty()
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
    /// Returns the square of the king with the specified color.
    ///
    /// # Panics
    /// Panics if the king is not present.
    pub fn get_king(&self, color: Color) -> Square {
        BitBoard::bit_scan_forward(self.get_color_piece(color, Piece::King))
            .expect("king does not exist")
    }
    /// Returns the pieces declaring check to the king of the specified color.
    ///
    /// # Panics
    /// Panics if the king is not present.
    pub fn get_king_checkers(&self, color: Color) -> BitBoard {
        self.get_color_attackers_to(self.get_king(color), !color)
    }
    /// Returns `true` if the king of the given color is currently in check.
    ///
    /// # Panics
    /// Panics if the king is not present.
    #[inline(always)]
    #[must_use]
    pub fn is_king_in_check(&self, color: Color) -> bool {
        !self.get_king_checkers(color).is_empty()
    }
    /// Places (or replaces) pieces of the given color on the squares
    /// specified by the mask with the given piece type.
    ///
    /// # Preconditions
    ///
    /// The user of the function is responsible for not trying to overwrite the squares that contain
    /// the opposite color, which will result in doubly colored pieces.
    #[inline(always)]
    pub fn mask_or(&mut self, color: Color, piece: Piece, mask: BitBoard) {
        self.pieces[piece as usize] |= mask;
        self.colors[color as usize] |= mask;
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
        self.pieces[piece as usize] &= mask;
        self.colors[color as usize] &= mask;
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
        self.pieces[piece as usize] ^= mask;
        self.colors[color as usize] ^= mask;
    }
    /// Returns an ASCII image of the board.
    ///
    /// Capitalized letters represent white pieces.
    /// - 'K' = king
    /// - 'Q' = queen
    /// - 'R' = rook
    /// - 'B' = bishop
    /// - 'N' = knight
    /// - 'P' = pawn
    pub fn ascii_image(&self) -> String {
        let mut img = String::new();
        for rank in Rank::iter().rev() {
            img.push(' ');
            for file in File::iter() {
                let sq = Square::new(rank, file);
                let color_piece = self.get_color_piece_at(sq);
                if let Some((color, piece)) = color_piece {
                    img.push(piece_to_fen(color, piece));
                } else {
                    img.push('_');
                }
                img += " ";
            }
            img += "\n"
        }
        img
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Chess board {{\n{}}}", self.ascii_image())
    }
}
