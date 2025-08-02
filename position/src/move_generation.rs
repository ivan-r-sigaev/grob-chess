use strum::{EnumCount, FromRepr, VariantArray};

use crate::{
    bitboard::BitBoard,
    indexing::{Color, File, Piece, Rank, Square},
    position::{CastlingRights, Position},
};

#[repr(u8)]
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr, EnumCount, VariantArray,
)]
pub enum ChessMoveHint {
    Quiet = 0,
    DoublePawn = 1,
    KingCastle = 2,
    QueenCastle = 3,
    Caputre = 4,
    EnPassantCapture = 5,
    KnightPromotion = 8,
    BishopPromotion = 9,
    RookPromotion = 10,
    QueenPromotion = 11,
    KnightPromotionCapture = 12,
    BishopPromotionCapture = 13,
    RookPromotionCapture = 14,
    QueenPromotionCapture = 15,
}

impl ChessMoveHint {
    #[inline(always)]
    #[must_use]
    pub fn is_capture(self) -> bool {
        self as u8 & 0b100 != 0
    }
    #[inline(always)]
    #[must_use]
    pub fn is_promotion(self) -> bool {
        self as u8 & 0b1000 != 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChessMove {
    pub to: Square,
    pub from: Square,
    pub hint: ChessMoveHint,
}

#[derive(Debug, Clone, Copy)]
pub struct ChessUnmove {
    chess_move: ChessMove,
    // Capture does not need to be wrapped, but it's safer.
    // Could maybe remove if it'll be bad for performance.
    capture: Option<Piece>,
    en_passant: Option<File>,
    castling_rights: CastlingRights,
    move_index_rule_50: u32,
}

impl Position {
    pub fn push_king_moves(&self, push_move: &mut impl FnMut(ChessMove)) {
        let from = self
            .board()
            .get_color_piece(self.turn(), Piece::King)
            .bit_scan_forward()
            .expect("king does not exist");
        let attacks = BitBoard::king_attacks(from);

        for to in self.board().get_color(!self.turn()) & attacks {
            push_move(ChessMove {
                from,
                to,
                hint: ChessMoveHint::Caputre,
            });
        }

        for to in !self.board().get_occupance() & attacks {
            push_move(ChessMove {
                from,
                to,
                hint: ChessMoveHint::Quiet,
            });
        }
    }
    pub fn push_knight_moves(&self, push_move: &mut impl FnMut(ChessMove)) {
        let opp = self.board().get_color(!self.turn());
        let empty = !self.board().get_occupance();

        for from in self.board().get_color_piece(self.turn(), Piece::Knight) {
            let attacks = BitBoard::knight_attacks(from);

            for to in attacks & opp {
                push_move(ChessMove {
                    from,
                    to,
                    hint: ChessMoveHint::Caputre,
                });
            }

            for to in attacks & empty {
                push_move(ChessMove {
                    from,
                    to,
                    hint: ChessMoveHint::Quiet,
                });
            }
        }
    }
    pub fn push_bishop_moves(&self, push_move: &mut impl FnMut(ChessMove)) {
        let opp = self.board().get_color(!self.turn());
        let occ = self.board().get_occupance();

        for from in self.board().get_color_bishop_sliders(self.turn()) {
            let attacks = BitBoard::bishop_attacks(occ, from);

            for to in attacks & opp {
                push_move(ChessMove {
                    from,
                    to,
                    hint: ChessMoveHint::Caputre,
                });
            }

            for to in attacks & !occ {
                push_move(ChessMove {
                    from,
                    to,
                    hint: ChessMoveHint::Quiet,
                });
            }
        }
    }
    pub fn push_rook_moves(&self, push_move: &mut impl FnMut(ChessMove)) {
        let opp = self.board().get_color(!self.turn());
        let occ = self.board().get_occupance();

        for from in self.board().get_color_rook_sliders(self.turn()) {
            let attacks = BitBoard::rook_attacks(occ, from);

            for to in attacks & opp {
                push_move(ChessMove {
                    from,
                    to,
                    hint: ChessMoveHint::Caputre,
                });
            }

            for to in attacks & !occ {
                push_move(ChessMove {
                    from,
                    to,
                    hint: ChessMoveHint::Quiet,
                });
            }
        }
    }
    pub fn push_pawn_quiets(&self, push_move: &mut impl FnMut(ChessMove)) {
        let empty = !self.board().get_occupance();
        let mut single_pushes = BitBoard::pawn_pushes(
            self.board().get_color_piece(self.turn(), Piece::Pawn),
            empty,
            self.turn(),
        );
        let double_pushes = BitBoard::pawn_pushes(
            single_pushes
                & if self.turn() == Color::White {
                    BitBoard::from_rank(Rank::R3)
                } else {
                    BitBoard::from_rank(Rank::R6)
                },
            empty,
            self.turn(),
        );
        let promotion_pushes = single_pushes
            & if self.turn() == Color::White {
                BitBoard::from_rank(Rank::R8)
            } else {
                BitBoard::from_rank(Rank::R1)
            };
        single_pushes &= !promotion_pushes;
        let push_offset = if self.turn() == Color::White { -8 } else { 8 };

        for to in single_pushes {
            push_move(ChessMove {
                from: to.shifted(push_offset),
                to,
                hint: ChessMoveHint::Quiet,
            });
        }

        for to in double_pushes {
            push_move(ChessMove {
                from: to.shifted(push_offset * 2),
                to,
                hint: ChessMoveHint::DoublePawn,
            });
        }

        for to in promotion_pushes {
            let from = to.shifted(push_offset);
            push_move(ChessMove {
                from,
                to,
                hint: ChessMoveHint::BishopPromotion,
            });
            push_move(ChessMove {
                from,
                to,
                hint: ChessMoveHint::KnightPromotion,
            });
            push_move(ChessMove {
                from,
                to,
                hint: ChessMoveHint::RookPromotion,
            });
            push_move(ChessMove {
                from,
                to,
                hint: ChessMoveHint::QueenPromotion,
            });
        }
    }
    pub fn push_pawn_attacks(&self, push_move: &mut impl FnMut(ChessMove)) {
        let opp = self.board().get_color(!self.turn());

        let mut pawns = self.board().get_color_piece(self.turn(), Piece::Pawn);
        let promoters = pawns
            & if self.turn() == Color::White {
                BitBoard::from_rank(Rank::R7)
            } else {
                BitBoard::from_rank(Rank::R2)
            };
        pawns &= !promoters;
        for from in promoters {
            for to in BitBoard::pawn_attacks(from, self.turn()) & opp {
                push_move(ChessMove {
                    from,
                    to,
                    hint: ChessMoveHint::BishopPromotionCapture,
                });
                push_move(ChessMove {
                    from,
                    to,
                    hint: ChessMoveHint::KnightPromotionCapture,
                });
                push_move(ChessMove {
                    from,
                    to,
                    hint: ChessMoveHint::RookPromotionCapture,
                });
                push_move(ChessMove {
                    from,
                    to,
                    hint: ChessMoveHint::QueenPromotionCapture,
                });
            }
        }

        if self.en_passant().is_some() {
            let to = Square::new(
                self.turn().en_passant_dest_rank(),
                self.en_passant().unwrap(),
            );
            for from in pawns & BitBoard::pawn_attacks(to, !self.turn()) {
                push_move(ChessMove {
                    from,
                    to,
                    hint: ChessMoveHint::EnPassantCapture,
                });
            }
        }

        for from in pawns {
            for to in BitBoard::pawn_attacks(from, self.turn()) & opp {
                push_move(ChessMove {
                    from,
                    to,
                    hint: ChessMoveHint::Caputre,
                });
            }
        }
    }
    pub fn push_castlings(&self, push_move: &mut impl FnMut(ChessMove)) {
        if self.board().is_king_in_check(self.turn()) {
            return;
        }

        if !self.is_kingside_castling_prohibited(self.turn()) {
            push_move(ChessMove {
                from: if self.turn() == Color::White {
                    Square::E1
                } else {
                    Square::E8
                },
                to: if self.turn() == Color::White {
                    Square::G1
                } else {
                    Square::G8
                },
                hint: ChessMoveHint::KingCastle,
            });
        }
        if !self.is_queenside_castling_prohibited(self.turn()) {
            push_move(ChessMove {
                from: if self.turn() == Color::White {
                    Square::E1
                } else {
                    Square::E8
                },
                to: if self.turn() == Color::White {
                    Square::C1
                } else {
                    Square::C8
                },
                hint: ChessMoveHint::QueenCastle,
            });
        }
    }
}

impl Position {
    #[must_use]
    pub fn can_make_move(&self, chess_move: ChessMove) -> bool {
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
                    && from.rank() == color.pawn_rank()
                    && BitBoard::pawn_pushes(
                        BitBoard::pawn_pushes(BitBoard::from(from), empty, color),
                        empty,
                        color,
                    ) == BitBoard::from(to)
            }
            ChessMoveHint::KingCastle => {
                piece == Piece::King && !self.is_kingside_castling_prohibited(color)
            }
            ChessMoveHint::QueenCastle => {
                piece == Piece::King && !self.is_queenside_castling_prohibited(color)
            }
            ChessMoveHint::EnPassantCapture => {
                let file = match self.en_passant() {
                    Some(file) => file,
                    None => return false,
                };
                let target_sq = Square::new(from.rank(), to.file());
                Square::new(color.en_passant_dest_rank(), file) == to
                    && !(BitBoard::pawn_attacks(from, color) & BitBoard::from(to)).is_empty()
                    && self.board().get_piece_at(target_sq) == Some(Piece::Pawn)
                    && self.board().get_color_at(target_sq) == Some(!color)
            }
            ChessMoveHint::KnightPromotion
            | ChessMoveHint::BishopPromotion
            | ChessMoveHint::RookPromotion
            | ChessMoveHint::QueenPromotion => {
                piece == Piece::Pawn
                    && to.rank() == color.promotion_rank()
                    && BitBoard::pawn_pushes(BitBoard::from(from), BitBoard::FILLED, color)
                        == BitBoard::from(to)
            }
            ChessMoveHint::KnightPromotionCapture
            | ChessMoveHint::BishopPromotionCapture
            | ChessMoveHint::RookPromotionCapture
            | ChessMoveHint::QueenPromotionCapture => {
                piece == Piece::Pawn
                    && to.rank() == color.promotion_rank()
                    && !(BitBoard::pawn_attacks(from, color) & BitBoard::from(to)).is_empty()
            }
        }
    }

    #[must_use]
    pub fn make_move(&mut self, chess_move: ChessMove) -> ChessUnmove {
        debug_assert!(self.can_make_move(chess_move), "buerak -> {chess_move:?}");

        let from = chess_move.from;
        let to = chess_move.to;
        let hint = chess_move.hint;

        let capture;
        let en_passant = self.en_passant();
        let castling_rights = self.castling_rights();
        let halfmove_index = self.move_index_rule_50();
        self.set_move_index(self.move_index() + 1);

        match hint {
            ChessMoveHint::Quiet => {
                let piece = self
                    .board()
                    .get_piece_at(from)
                    .expect("faulty move concept: from is empty");

                capture = None;
                self.set_en_passant(None);
                if piece == Piece::Pawn {
                    self.set_move_index_rule_50(self.move_index());
                }

                if piece == Piece::King {
                    self.set_castling_rights(
                        self.castling_rights() & !CastlingRights::both_sides(self.turn()),
                    );
                } else if piece == Piece::Rook {
                    if from
                        == (if self.turn() == Color::White {
                            Square::H1
                        } else {
                            Square::H8
                        })
                    {
                        self.set_castling_rights(
                            self.castling_rights() & !CastlingRights::kingside(self.turn()),
                        );
                    } else if from
                        == (if self.turn() == Color::White {
                            Square::A1
                        } else {
                            Square::A8
                        })
                    {
                        self.set_castling_rights(
                            self.castling_rights() & !CastlingRights::queenside(self.turn()),
                        );
                    }
                }

                self.remove_color_piece(self.turn(), piece, from);
                self.add_color_piece(self.turn(), piece, to);
            }
            ChessMoveHint::DoublePawn => {
                capture = None;
                self.set_en_passant(Some(from.file()));
                self.set_move_index_rule_50(self.move_index());

                self.remove_color_piece(self.turn(), Piece::Pawn, from);
                self.add_color_piece(self.turn(), Piece::Pawn, to);
            }
            ChessMoveHint::BishopPromotion => {
                capture = None;
                self.set_en_passant(None);
                self.set_move_index_rule_50(self.move_index());

                self.remove_color_piece(self.turn(), Piece::Pawn, from);
                self.add_color_piece(self.turn(), Piece::Bishop, to);
            }
            ChessMoveHint::KnightPromotion => {
                capture = None;
                self.set_en_passant(None);
                self.set_move_index_rule_50(self.move_index());

                self.remove_color_piece(self.turn(), Piece::Pawn, from);
                self.add_color_piece(self.turn(), Piece::Knight, to);
            }
            ChessMoveHint::RookPromotion => {
                capture = None;
                self.set_en_passant(None);
                self.set_move_index_rule_50(self.move_index());

                self.remove_color_piece(self.turn(), Piece::Pawn, from);
                self.add_color_piece(self.turn(), Piece::Rook, to);
            }
            ChessMoveHint::QueenPromotion => {
                capture = None;
                self.set_en_passant(None);
                self.set_move_index_rule_50(self.move_index());

                self.remove_color_piece(self.turn(), Piece::Pawn, from);
                self.add_color_piece(self.turn(), Piece::Queen, to);
            }
            ChessMoveHint::Caputre => {
                let piece = self
                    .board()
                    .get_piece_at(from)
                    .expect("faulty move concept: from is empty");
                let captured_piece = self
                    .board()
                    .get_piece_at(to)
                    .expect("faulty move concept: to is empty");

                capture = Some(captured_piece);
                self.set_en_passant(None);

                if piece == Piece::King {
                    self.set_castling_rights(
                        self.castling_rights() & !CastlingRights::both_sides(self.turn()),
                    );
                } else if piece == Piece::Rook {
                    if from
                        == (if self.turn() == Color::White {
                            Square::H1
                        } else {
                            Square::H8
                        })
                    {
                        self.set_castling_rights(
                            self.castling_rights() & !CastlingRights::kingside(self.turn()),
                        );
                    } else if from
                        == (if self.turn() == Color::White {
                            Square::A1
                        } else {
                            Square::A8
                        })
                    {
                        self.set_castling_rights(
                            self.castling_rights() & !CastlingRights::queenside(self.turn()),
                        );
                    }
                }

                self.remove_color_piece(self.turn(), piece, from);
                self.remove_color_piece(!self.turn(), captured_piece, to);
                self.add_color_piece(self.turn(), piece, to);
            }
            ChessMoveHint::EnPassantCapture => {
                capture = Some(Piece::Pawn);
                self.set_en_passant(None);
                self.set_move_index_rule_50(self.move_index());

                self.remove_color_piece(self.turn(), Piece::Pawn, from);
                self.remove_color_piece(
                    !self.turn(),
                    Piece::Pawn,
                    Square::new(from.rank(), to.file()),
                );
                self.add_color_piece(self.turn(), Piece::Pawn, to);
            }
            ChessMoveHint::BishopPromotionCapture => {
                let captured_piece = self
                    .board()
                    .get_piece_at(to)
                    .expect("faulty move concept: to is empty");

                capture = Some(captured_piece);
                self.set_en_passant(None);
                self.set_move_index_rule_50(self.move_index());

                self.remove_color_piece(self.turn(), Piece::Pawn, from);
                self.remove_color_piece(!self.turn(), captured_piece, to);
                self.add_color_piece(self.turn(), Piece::Bishop, to);
            }
            ChessMoveHint::KnightPromotionCapture => {
                let captured_piece = self
                    .board()
                    .get_piece_at(to)
                    .expect("faulty move concept: to is empty");

                capture = Some(captured_piece);
                self.set_en_passant(None);
                self.set_move_index_rule_50(self.move_index());

                self.remove_color_piece(self.turn(), Piece::Pawn, from);
                self.remove_color_piece(!self.turn(), captured_piece, to);
                self.add_color_piece(self.turn(), Piece::Knight, to);
            }
            ChessMoveHint::RookPromotionCapture => {
                let captured_piece = self
                    .board()
                    .get_piece_at(to)
                    .expect("faulty move concept: to is empty");

                capture = Some(captured_piece);
                self.set_en_passant(None);
                self.set_move_index_rule_50(self.move_index());

                self.remove_color_piece(self.turn(), Piece::Pawn, from);
                self.remove_color_piece(!self.turn(), captured_piece, to);
                self.add_color_piece(self.turn(), Piece::Rook, to);
            }
            ChessMoveHint::QueenPromotionCapture => {
                let captured_piece = self
                    .board()
                    .get_piece_at(to)
                    .expect("faulty move concept: to is empty");

                capture = Some(captured_piece);
                self.set_en_passant(None);
                self.set_move_index_rule_50(self.move_index());

                self.remove_color_piece(self.turn(), Piece::Pawn, from);
                self.remove_color_piece(!self.turn(), captured_piece, to);
                self.add_color_piece(self.turn(), Piece::Queen, to);
            }
            ChessMoveHint::KingCastle => {
                capture = None;
                self.set_en_passant(None);
                self.set_castling_rights(
                    self.castling_rights() & !CastlingRights::both_sides(self.turn()),
                );

                self.remove_color_piece(self.turn(), Piece::King, from);
                self.add_color_piece(self.turn(), Piece::King, to);

                self.remove_color_piece(
                    self.turn(),
                    Piece::Rook,
                    if self.turn() == Color::White {
                        Square::H1
                    } else {
                        Square::H8
                    },
                );
                self.add_color_piece(
                    self.turn(),
                    Piece::Rook,
                    if self.turn() == Color::White {
                        Square::F1
                    } else {
                        Square::F8
                    },
                );
            }
            ChessMoveHint::QueenCastle => {
                capture = None;
                self.set_en_passant(None);
                self.set_castling_rights(
                    self.castling_rights() & !CastlingRights::both_sides(self.turn()),
                );

                self.remove_color_piece(self.turn(), Piece::King, from);
                self.add_color_piece(self.turn(), Piece::King, to);

                self.remove_color_piece(
                    self.turn(),
                    Piece::Rook,
                    if self.turn() == Color::White {
                        Square::A1
                    } else {
                        Square::A8
                    },
                );
                self.add_color_piece(
                    self.turn(),
                    Piece::Rook,
                    if self.turn() == Color::White {
                        Square::D1
                    } else {
                        Square::D8
                    },
                );
            }
        }

        self.set_turn(!self.turn());

        ChessUnmove {
            chess_move,
            capture,
            en_passant,
            castling_rights,
            move_index_rule_50: halfmove_index,
        }
    }

    pub fn unmake_move(&mut self, unmove_concept: ChessUnmove) {
        self.set_turn(!self.turn());
        self.set_castling_rights(unmove_concept.castling_rights);
        self.set_en_passant(unmove_concept.en_passant);
        self.set_move_index_rule_50(unmove_concept.move_index_rule_50);
        self.set_move_index(self.move_index() - 1);

        let from = unmove_concept.chess_move.from;
        let to = unmove_concept.chess_move.to;
        let hint = unmove_concept.chess_move.hint;

        match hint {
            ChessMoveHint::Quiet => {
                let piece = self
                    .board()
                    .get_piece_at(to)
                    .expect("faulty unmove concept: to is empty");

                self.remove_color_piece(self.turn(), piece, to);
                self.add_color_piece(self.turn(), piece, from);
            }
            ChessMoveHint::DoublePawn => {
                self.remove_color_piece(self.turn(), Piece::Pawn, to);
                self.add_color_piece(self.turn(), Piece::Pawn, from);
            }
            ChessMoveHint::KnightPromotion => {
                self.remove_color_piece(self.turn(), Piece::Knight, to);
                self.add_color_piece(self.turn(), Piece::Pawn, from);
            }
            ChessMoveHint::BishopPromotion => {
                self.remove_color_piece(self.turn(), Piece::Bishop, to);
                self.add_color_piece(self.turn(), Piece::Pawn, from);
            }
            ChessMoveHint::RookPromotion => {
                self.remove_color_piece(self.turn(), Piece::Rook, to);
                self.add_color_piece(self.turn(), Piece::Pawn, from);
            }
            ChessMoveHint::QueenPromotion => {
                self.remove_color_piece(self.turn(), Piece::Queen, to);
                self.add_color_piece(self.turn(), Piece::Pawn, from);
            }
            ChessMoveHint::Caputre => {
                let piece = self
                    .board()
                    .get_piece_at(to)
                    .expect("faulty unmove concept: to is empty");
                let captured_piece = unmove_concept
                    .capture
                    .expect("faulty unmove concept: no captured piece");

                self.remove_color_piece(self.turn(), piece, to);
                self.add_color_piece(!self.turn(), captured_piece, to);
                self.add_color_piece(self.turn(), piece, from);
            }
            ChessMoveHint::EnPassantCapture => {
                self.remove_color_piece(self.turn(), Piece::Pawn, to);
                self.add_color_piece(
                    !self.turn(),
                    Piece::Pawn,
                    Square::new(from.rank(), to.file()),
                );
                self.add_color_piece(self.turn(), Piece::Pawn, from);
            }
            ChessMoveHint::KnightPromotionCapture => {
                let captured_piece = unmove_concept
                    .capture
                    .expect("faulty unmove concept: no captured piece");

                self.remove_color_piece(self.turn(), Piece::Knight, to);
                self.add_color_piece(!self.turn(), captured_piece, to);
                self.add_color_piece(self.turn(), Piece::Pawn, from);
            }
            ChessMoveHint::BishopPromotionCapture => {
                let captured_piece = unmove_concept
                    .capture
                    .expect("faulty unmove concept: no captured piece");

                self.remove_color_piece(self.turn(), Piece::Bishop, to);
                self.add_color_piece(!self.turn(), captured_piece, to);
                self.add_color_piece(self.turn(), Piece::Pawn, from);
            }
            ChessMoveHint::RookPromotionCapture => {
                let captured_piece = unmove_concept
                    .capture
                    .expect("faulty unmove concept: no captured piece");

                self.remove_color_piece(self.turn(), Piece::Rook, to);
                self.add_color_piece(!self.turn(), captured_piece, to);
                self.add_color_piece(self.turn(), Piece::Pawn, from);
            }
            ChessMoveHint::QueenPromotionCapture => {
                let captured_piece = unmove_concept
                    .capture
                    .expect("faulty unmove concept: no captured piece");

                self.remove_color_piece(self.turn(), Piece::Queen, to);
                self.add_color_piece(!self.turn(), captured_piece, to);
                self.add_color_piece(self.turn(), Piece::Pawn, from);
            }
            ChessMoveHint::KingCastle => {
                self.remove_color_piece(self.turn(), Piece::King, to);
                self.add_color_piece(self.turn(), Piece::King, from);

                self.remove_color_piece(
                    self.turn(),
                    Piece::Rook,
                    if self.turn() == Color::White {
                        Square::F1
                    } else {
                        Square::F8
                    },
                );
                self.add_color_piece(
                    self.turn(),
                    Piece::Rook,
                    if self.turn() == Color::White {
                        Square::H1
                    } else {
                        Square::H8
                    },
                );
            }
            ChessMoveHint::QueenCastle => {
                self.remove_color_piece(self.turn(), Piece::King, to);
                self.add_color_piece(self.turn(), Piece::King, from);

                self.remove_color_piece(
                    self.turn(),
                    Piece::Rook,
                    if self.turn() == Color::White {
                        Square::D1
                    } else {
                        Square::D8
                    },
                );
                self.add_color_piece(
                    self.turn(),
                    Piece::Rook,
                    if self.turn() == Color::White {
                        Square::A1
                    } else {
                        Square::A8
                    },
                );
            }
        }
    }
}

impl Position {
    /// Returns whether kingside castling is NOT allowed for a given color.
    ///
    /// # Returns
    /// `bool` - whether kingside castling is NOT allowed for a given color
    #[inline(always)]
    #[must_use]
    fn is_kingside_castling_prohibited(&self, color: Color) -> bool {
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
    fn is_queenside_castling_prohibited(&self, color: Color) -> bool {
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
