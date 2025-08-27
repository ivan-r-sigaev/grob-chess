use std::fmt;

use strum::{EnumCount, FromRepr, VariantArray};

use crate::{
    game::position::{lan::LanMove, Position},
    BitBoard, CastlingRights, Color, Piece, Promotion, Rank, Square,
};

/// A hint specifying what kind of move to perform.
#[repr(u8)]
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr, EnumCount, VariantArray,
)]
pub enum ChessMoveHint {
    /// Normal quiet move.
    Quiet = 0,
    /// Pawn push that moves two squares.
    DoublePawn = 1,
    /// Kingside castling.
    KingCastle = 2,
    /// Queenside castling.
    QueenCastle = 3,
    /// Normal capture move.
    Caputre = 4,
    /// En passant.
    EnPassantCapture = 5,
    /// Quiet promotion to knight.
    KnightPromotion = 8,
    /// Quiet promotion to bishop.
    BishopPromotion = 9,
    /// Quiet promotion to rook.
    RookPromotion = 10,
    /// Quiet promotion to queen.
    QueenPromotion = 11,
    /// Promotion to knight with capture.
    KnightPromotionCapture = 12,
    /// Promotion to bishop with capture.
    BishopPromotionCapture = 13,
    /// Promotion to rook with capture.
    RookPromotionCapture = 14,
    /// Promotion to queen with capture.
    QueenPromotionCapture = 15,
}

impl ChessMoveHint {
    /// Is this move a capture.
    #[inline(always)]
    #[must_use]
    pub fn is_capture(self) -> bool {
        self as u8 & 0b100 != 0
    }
    /// Is this move a promotion.
    #[inline(always)]
    #[must_use]
    pub fn is_promotion(self) -> bool {
        self as u8 & 0b1000 != 0
    }
    /// Returns the kind of promotion if this move is a promotion.
    pub fn promotion(self) -> Option<Promotion> {
        Some(match self {
            Self::BishopPromotion | Self::BishopPromotionCapture => Promotion::Bishop,
            Self::KnightPromotion | Self::KnightPromotionCapture => Promotion::Knight,
            Self::RookPromotion | Self::RookPromotionCapture => Promotion::Rook,
            Self::QueenPromotion | Self::QueenPromotionCapture => Promotion::Queen,
            _ => return None,
        })
    }
}

/// Data needed to make a move.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChessMove {
    to: Square,
    from: Square,
    hint: ChessMoveHint,
}

impl ChessMove {
    /// Returns the destination square of a move.
    pub fn dest_square(self) -> Square {
        self.to
    }
    /// Returns the origin square of a move.
    pub fn orig_square(self) -> Square {
        self.from
    }
    /// Returns the hint as to what kind of move is happening.
    pub fn hint(self) -> ChessMoveHint {
        self.hint
    }
    /// Converts the move to [`LanMove`].
    pub fn lan(self) -> LanMove {
        LanMove {
            to: self.to,
            from: self.from,
            promotion: self.hint.promotion(),
        }
    }
}

/// Compact version of a [`ChessMove`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PackedChessMove {
    data: u16,
}

impl PackedChessMove {
    /// Converts [`ChessMove`] to it's compact form.
    #[inline(always)]
    #[must_use]
    pub fn new(chess_move: ChessMove) -> Self {
        Self {
            data: (((chess_move.hint as u16) & 0xf) << 12)
                | (((chess_move.from as u16) & 0x3f) << 6)
                | ((chess_move.to as u16) & 0x3f),
        }
    }
    /// Unpacks the [`ChessMove`] from it's compact form.
    #[inline(always)]
    #[must_use]
    pub fn get(self) -> ChessMove {
        let to = Square::from_repr((self.data & 0x3f) as u8).unwrap();
        let from = Square::from_repr(((self.data >> 6) & 0x3f) as u8).unwrap();
        let hint = ChessMoveHint::from_repr(((self.data >> 12) & 0x0f) as u8).unwrap();
        ChessMove { to, from, hint }
    }
}

impl Position {
    /// Generate pseudo-legal moves from this position.
    pub fn push_moves(&self, push_move: &mut impl FnMut(ChessMove)) {
        if self.board().get_king_checkers(self.turn()).count() >= 2 {
            self.push_king_attacks(push_move);
            self.push_king_quiets(push_move);
            return;
        }

        self.push_pawn_attacks(push_move);
        self.push_knight_attacks(push_move);
        self.push_bishop_attacks(push_move);
        self.push_rook_attacks(push_move);
        self.push_king_attacks(push_move);

        self.push_castlings(push_move);
        self.push_king_quiets(push_move);
        self.push_rook_quiets(push_move);
        self.push_bishop_quiets(push_move);
        self.push_knight_quiets(push_move);
        self.push_pawn_quiets(push_move);
    }
    /// Generate pseudo-legal king's quiet moves from this position.
    pub fn push_king_quiets(&self, push_move: &mut impl FnMut(ChessMove)) {
        let from = self
            .board()
            .get_color_piece(self.turn(), Piece::King)
            .bit_scan_forward()
            .unwrap();
        let attacks = BitBoard::king_attacks(from);
        self.push_quiets(push_move, attacks, from, ChessMoveHint::Quiet);
    }
    /// Generate pseudo-legal king attacks from this position.
    pub fn push_king_attacks(&self, push_move: &mut impl FnMut(ChessMove)) {
        let from = self
            .board()
            .get_color_piece(self.turn(), Piece::King)
            .bit_scan_forward()
            .unwrap();
        let attacks = BitBoard::king_attacks(from);
        self.push_attacks(push_move, attacks, from, ChessMoveHint::Caputre);
    }
    /// Generate pseudo-legal knight's quiet moves from this position.
    pub fn push_knight_quiets(&self, push_move: &mut impl FnMut(ChessMove)) {
        for from in self.board().get_color_piece(self.turn(), Piece::Knight) {
            let attacks = BitBoard::knight_attacks(from);
            self.push_quiets(push_move, attacks, from, ChessMoveHint::Quiet);
        }
    }
    /// Generate pseudo-legal knight attacks from this position.
    pub fn push_knight_attacks(&self, push_move: &mut impl FnMut(ChessMove)) {
        for from in self.board().get_color_piece(self.turn(), Piece::Knight) {
            let attacks = BitBoard::knight_attacks(from);
            self.push_attacks(push_move, attacks, from, ChessMoveHint::Caputre);
        }
    }
    /// Generate pseudo-legal bishop-like quiet moves from this position.
    pub fn push_bishop_quiets(&self, push_move: &mut impl FnMut(ChessMove)) {
        let occ = self.board().get_occupance();

        for from in self.board().get_color_bishop_sliders(self.turn()) {
            let attacks = BitBoard::bishop_attacks(occ, from);
            self.push_quiets(push_move, attacks, from, ChessMoveHint::Quiet);
        }
    }
    /// Generate pseudo-legal bishop-like attacks from this position.
    pub fn push_bishop_attacks(&self, push_move: &mut impl FnMut(ChessMove)) {
        let occ = self.board().get_occupance();

        for from in self.board().get_color_bishop_sliders(self.turn()) {
            let attacks = BitBoard::bishop_attacks(occ, from);
            self.push_attacks(push_move, attacks, from, ChessMoveHint::Caputre);
        }
    }
    /// Generate pseudo-legal rook-like quiet moves from this position.
    pub fn push_rook_quiets(&self, push_move: &mut impl FnMut(ChessMove)) {
        let occ = self.board().get_occupance();

        for from in self.board().get_color_rook_sliders(self.turn()) {
            let attacks = BitBoard::rook_attacks(occ, from);
            self.push_quiets(push_move, attacks, from, ChessMoveHint::Quiet);
        }
    }
    /// Generate pseudo-legal rook-like attacks from this position.
    pub fn push_rook_attacks(&self, push_move: &mut impl FnMut(ChessMove)) {
        let occ = self.board().get_occupance();

        for from in self.board().get_color_rook_sliders(self.turn()) {
            let attacks = BitBoard::rook_attacks(occ, from);
            self.push_attacks(push_move, attacks, from, ChessMoveHint::Caputre);
        }
    }
    /// Generate pseudo-legal quiet pawn moves from this position.
    pub fn push_pawn_quiets(&self, push_move: &mut impl FnMut(ChessMove)) {
        let empty = !self.board().get_occupance();

        let single_pushes = BitBoard::pawn_pushes(
            self.board().get_color_piece(self.turn(), Piece::Pawn),
            empty,
            self.turn(),
        );
        let double_pushes = BitBoard::pawn_pushes(
            single_pushes
                & match self.turn() {
                    Color::White => BitBoard::from_rank(Rank::R3),
                    Color::Black => BitBoard::from_rank(Rank::R6),
                },
            empty,
            self.turn(),
        );
        let promotion_pushes = single_pushes
            & match self.turn() {
                Color::White => BitBoard::from_rank(Rank::R8),
                Color::Black => BitBoard::from_rank(Rank::R1),
            };
        let quiet_pushes = single_pushes & !promotion_pushes;
        let push_offset = if self.turn() == Color::White { -8 } else { 8 };

        Self::push_for_each_shifted(push_move, quiet_pushes, push_offset, ChessMoveHint::Quiet);
        Self::push_for_each_shifted(
            push_move,
            double_pushes,
            push_offset * 2,
            ChessMoveHint::DoublePawn,
        );
        Self::push_for_each_shifted(
            push_move,
            promotion_pushes,
            push_offset,
            ChessMoveHint::BishopPromotion,
        );
        Self::push_for_each_shifted(
            push_move,
            promotion_pushes,
            push_offset,
            ChessMoveHint::KnightPromotion,
        );
        Self::push_for_each_shifted(
            push_move,
            promotion_pushes,
            push_offset,
            ChessMoveHint::RookPromotion,
        );
        Self::push_for_each_shifted(
            push_move,
            promotion_pushes,
            push_offset,
            ChessMoveHint::QueenPromotion,
        );
    }
    /// Generate pseudo-legal pawn captures from this position.
    pub fn push_pawn_attacks(&self, push_move: &mut impl FnMut(ChessMove)) {
        let opp = self.board().get_color(!self.turn());

        let pawns = self.board().get_color_piece(self.turn(), Piece::Pawn);
        let promoters = pawns & BitBoard::from(self.turn().mirror_rank(Rank::R7));
        let pawns = pawns & !promoters;
        for from in promoters {
            let attacks = BitBoard::pawn_attacks(from, self.turn()) & opp;
            self.push_attacks(
                push_move,
                attacks,
                from,
                ChessMoveHint::BishopPromotionCapture,
            );
            self.push_attacks(
                push_move,
                attacks,
                from,
                ChessMoveHint::KnightPromotionCapture,
            );
            self.push_attacks(
                push_move,
                attacks,
                from,
                ChessMoveHint::RookPromotionCapture,
            );
            self.push_attacks(
                push_move,
                attacks,
                from,
                ChessMoveHint::QueenPromotionCapture,
            );
        }

        if let Some(file) = self.en_passant() {
            let to = Square::new(
                {
                    let this = self.turn();
                    this.mirror_rank(Rank::R6)
                },
                file,
            );
            for from in pawns & BitBoard::pawn_attacks(to, !self.turn()) {
                let hint = ChessMoveHint::EnPassantCapture;
                push_move(ChessMove { from, to, hint });
            }
        }

        for from in pawns {
            let attacks = BitBoard::pawn_attacks(from, self.turn()) & opp;
            self.push_attacks(push_move, attacks, from, ChessMoveHint::Caputre);
        }
    }
    /// Generate pseudo-legal castling moves from this position.
    pub fn push_castlings(&self, push_move: &mut impl FnMut(ChessMove)) {
        if self.board().is_king_in_check(self.turn()) {
            return;
        }

        if self.is_kingside_castling_allowed(self.turn()) {
            push_move(ChessMove {
                from: self.turn().mirror_square(Square::E1),
                to: self.turn().mirror_square(Square::G1),
                hint: ChessMoveHint::KingCastle,
            });
        }
        if self.is_queenside_castling_allowed(self.turn()) {
            push_move(ChessMove {
                from: self.turn().mirror_square(Square::E1),
                to: self.turn().mirror_square(Square::C1),
                hint: ChessMoveHint::QueenCastle,
            });
        }
    }
    fn push_attacks(
        &self,
        push_move: &mut impl FnMut(ChessMove),
        attacks: BitBoard,
        from: Square,
        hint: ChessMoveHint,
    ) {
        let board = self.board();
        let color = !self.turn();
        Self::push_for_each(
            push_move,
            attacks & board.get_color_piece(color, Piece::Queen),
            from,
            hint,
        );
        Self::push_for_each(
            push_move,
            attacks & board.get_color_piece(color, Piece::Rook),
            from,
            hint,
        );
        Self::push_for_each(
            push_move,
            attacks & board.get_color_piece(color, Piece::Bishop),
            from,
            hint,
        );
        Self::push_for_each(
            push_move,
            attacks & board.get_color_piece(color, Piece::Knight),
            from,
            hint,
        );
        Self::push_for_each(
            push_move,
            attacks & board.get_color_piece(color, Piece::Pawn),
            from,
            hint,
        );
    }
    fn push_quiets(
        &self,
        push_move: &mut impl FnMut(ChessMove),
        attacks: BitBoard,
        from: Square,
        hint: ChessMoveHint,
    ) {
        let board = self.board();
        Self::push_for_each(push_move, attacks & !board.get_occupance(), from, hint);
    }
    fn push_for_each(
        push_move: &mut impl FnMut(ChessMove),
        to: BitBoard,
        from: Square,
        hint: ChessMoveHint,
    ) {
        for to in to {
            push_move(ChessMove { to, from, hint });
        }
    }
    fn push_for_each_shifted(
        push_move: &mut impl FnMut(ChessMove),
        to: BitBoard,
        delta: i8,
        hint: ChessMoveHint,
    ) {
        for to in to {
            let from = to.shifted(delta);
            push_move(ChessMove { to, from, hint });
        }
    }
    /// Returns whether the kingside castling is allowed for a given color.
    #[inline(always)]
    #[must_use]
    fn is_kingside_castling_allowed(&self, color: Color) -> bool {
        self.castling_rights()
            .contains(CastlingRights::kingside(color))
            && !self.board().is_king_in_check(color)
            && self
                .board()
                .can_king_move_to(color.mirror_square(Square::F1), color)
            && self
                .board()
                .can_king_move_to(color.mirror_square(Square::G1), color)
    }
    /// Returns whether the queenside castling is allowed for a given color.
    #[inline(always)]
    #[must_use]
    fn is_queenside_castling_allowed(&self, color: Color) -> bool {
        self.castling_rights()
            .contains(CastlingRights::queenside(color))
            && !self.board().is_king_in_check(color)
            && !self
                .board()
                .get_occupance()
                .has_square(color.mirror_square(Square::B1))
            && self
                .board()
                .can_king_move_to(color.mirror_square(Square::C1), color)
            && self
                .board()
                .can_king_move_to(color.mirror_square(Square::D1), color)
    }
    /// Returns whether a given chess move is at least pseudo-legal in this position.
    #[must_use]
    pub fn is_move_pseudo_legal(&self, chess_move: ChessMove) -> bool {
        let from = chess_move.from;
        let to = chess_move.to;
        let hint = chess_move.hint;

        let piece = match self.board().get_piece_at(from) {
            Some(piece) => piece,
            None => return false,
        };

        let color = match self.board().get_color_at(from) {
            Some(color) => color,
            None => return false,
        };
        if color != self.turn() {
            return false;
        }

        let occ = self.board().get_occupance();
        let empty = self.board().get_empty();
        let target = self.board().get_piece_at(to);
        let target_color = self.board().get_color_at(to);

        if let Some(tgt_color) = target_color {
            if tgt_color == color {
                return false;
            }
        }

        if target.is_some() != hint.is_capture() && hint != ChessMoveHint::EnPassantCapture {
            return false;
        }

        match hint {
            ChessMoveHint::Quiet | ChessMoveHint::Caputre => match piece {
                Piece::Pawn => {
                    if piece != Piece::Pawn {
                        return false;
                    }
                    if hint.is_capture() {
                        !(BitBoard::pawn_attacks(from, color) & BitBoard::from(to)).is_empty()
                    } else {
                        !(BitBoard::pawn_pushes(BitBoard::from(from), empty, color)
                            & BitBoard::from(to))
                        .is_empty()
                    }
                }
                Piece::Bishop => {
                    !(BitBoard::bishop_attacks(occ, from) & BitBoard::from(to)).is_empty()
                }
                Piece::Knight => !(BitBoard::knight_attacks(from) & BitBoard::from(to)).is_empty(),
                Piece::Rook => !(BitBoard::rook_attacks(occ, from) & BitBoard::from(to)).is_empty(),
                Piece::Queen => {
                    !(BitBoard::queen_attacks(occ, from) & BitBoard::from(to)).is_empty()
                }
                Piece::King => !(BitBoard::king_attacks(from) & BitBoard::from(to)).is_empty(),
            },
            ChessMoveHint::DoublePawn => {
                piece == Piece::Pawn
                    && from.rank() == color.mirror_rank(Rank::R2)
                    && BitBoard::pawn_pushes(
                        BitBoard::pawn_pushes(BitBoard::from(from), empty, color),
                        empty,
                        color,
                    ) == BitBoard::from(to)
            }
            ChessMoveHint::KingCastle => {
                piece == Piece::King && self.is_kingside_castling_allowed(color)
            }
            ChessMoveHint::QueenCastle => {
                piece == Piece::King && self.is_queenside_castling_allowed(color)
            }
            ChessMoveHint::EnPassantCapture => {
                let file = match self.en_passant() {
                    Some(file) => file,
                    None => return false,
                };
                let target_sq = Square::new(from.rank(), to.file());
                Square::new(color.mirror_rank(Rank::R6), file) == to
                    && !(BitBoard::pawn_attacks(from, color) & BitBoard::from(to)).is_empty()
                    && self.board().get_piece_at(target_sq) == Some(Piece::Pawn)
                    && self.board().get_color_at(target_sq) == Some(!color)
            }
            ChessMoveHint::KnightPromotion
            | ChessMoveHint::BishopPromotion
            | ChessMoveHint::RookPromotion
            | ChessMoveHint::QueenPromotion => {
                piece == Piece::Pawn
                    && to.rank() == color.mirror_rank(Rank::R8)
                    && BitBoard::pawn_pushes(BitBoard::from(from), BitBoard::FILLED, color)
                        == BitBoard::from(to)
            }
            ChessMoveHint::KnightPromotionCapture
            | ChessMoveHint::BishopPromotionCapture
            | ChessMoveHint::RookPromotionCapture
            | ChessMoveHint::QueenPromotionCapture => {
                piece == Piece::Pawn
                    && to.rank() == color.mirror_rank(Rank::R8)
                    && !(BitBoard::pawn_attacks(from, color) & BitBoard::from(to)).is_empty()
            }
        }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            concat!(
                // TODO: this does not display history
                "Chess position {{\n",
                "  turn: {}\n",
                "  castling rights: {}\n",
                "  available en passant: {}\n",
                "  moves since last capture/pawn move: {}\n",
                "  hash: {}\n",
                "  board: {}\n}}"
            ),
            self.turn(),
            self.castling_rights(),
            if let Some(en_passant) = self.en_passant() {
                &format!("on {en_passant} file")
            } else {
                "N/A"
            },
            self.move_index() - self.move_index_rule_50(),
            self.zobrist(),
            self.board(),
        )
    }
}
