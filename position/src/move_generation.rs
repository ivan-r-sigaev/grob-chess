use crate::{board::{Color, Piece}, bitboard::{BitBoard, Square}, position::{Position, CastlingRights}};

impl Position {
    /// Returns whether kingside castling is NOT allowed for a given color.
    ///
    /// # Returns
    /// `bool` - whether kingside castling is NOT allowed for a given color
    #[inline(always)]
    #[must_use]
    pub fn is_kingside_castling_prohibited(&self, color: Color) -> bool {
        // TODO: remove crights when rook is taken instead of checking for it's existence
        let w_empty = BitBoard::from(Square::F1) | BitBoard::from(Square::G1);
        let b_empty = BitBoard::from(Square::F8) | BitBoard::from(Square::G8);
        !self
            .castling_rights()
            .contains(CastlingRights::kingside(self.turn()))
            || (self.board().get_color_piece(color, Piece::Rook)
                & BitBoard::from(if color == Color::White {
                    Square::H1
                } else {
                    Square::H8
                }))
            .is_empty()
            || !(self.board().get_occupance()
                & if color == Color::White {
                    w_empty
                } else {
                    b_empty
                })
            .is_empty()
            || !(self.board().get_color_attackers_to(
                if color == Color::White {
                    Square::F1
                } else {
                    Square::F8
                },
                !color,
            ))
            .is_empty()
            || !(self.board().get_color_attackers_to(
                if color == Color::White {
                    Square::G1
                } else {
                    Square::G8
                },
                !color,
            ))
            .is_empty()
    }

    /// Returns whether queenside castling is NOT allowed for a given color.
    ///
    /// # Returns
    /// `bool` - whether queenside castling is NOT allowed for a given color
    #[inline(always)]
    #[must_use]
    pub fn is_queenside_castling_prohibited(&self, color: Color) -> bool {
        // TODO: remove crights when rook is taken instead of checking for it's existence
        let w_empty =
            BitBoard::from(Square::B1) | BitBoard::from(Square::C1) | BitBoard::from(Square::D1);
        let b_empty =
            BitBoard::from(Square::B8) | BitBoard::from(Square::C8) | BitBoard::from(Square::D8);
        !self
            .castling_rights()
            .contains(CastlingRights::queenside(self.turn()))
            || (self.board().get_color_piece(color, Piece::Rook)
                & BitBoard::from(if color == Color::White {
                    Square::A1
                } else {
                    Square::A8
                }))
            .is_empty()
            || !(self.board().get_occupance()
                & if color == Color::White {
                    w_empty
                } else {
                    b_empty
                })
            .is_empty()
            || !(self.board().get_color_attackers_to(
                if color == Color::White {
                    Square::C1
                } else {
                    Square::C8
                },
                !color,
            ))
            .is_empty()
            || !(self.board().get_color_attackers_to(
                if color == Color::White {
                    Square::D1
                } else {
                    Square::D8
                },
                !color,
            ))
            .is_empty()
    }
}
