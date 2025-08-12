use crate::board::{BitBoard, Color, File, Piece, Promotion, Rank, Square};
use crate::position::{CastlingRights, Position};
use std::{fmt, str::FromStr};
use strum::{EnumCount, FromRepr, VariantArray};

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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

/// Data needed to rollback a move.
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
    /// Generate pseudo-legal king moves from this position.
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
    /// Generate pseudo-legal knight moves from this position.
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
    /// Generate pseudo-legal bishop-like moves from this position.
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
    /// Generate pseudo-legal rook-like moves from this position.
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
    /// Generate pseudo-legal quiet pawn moves from this position.
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
    /// Generate pseudo-legal pawn captures from this position.
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
    /// Generate pseudo-legal castling moves from this position.
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

/// A chess move in a [LAN (Long Algebraic Notation)].
///
/// [LAN (Long Algebraic Notation)]:
/// https://www.chessprogramming.org/Algebraic_Chess_Notation#Long_Algebraic_Notation_.28LAN.29
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LanMove {
    /// The origin square.
    pub from: Square,
    /// The destination square.
    pub to: Square,
    /// The kind of promotion (if move is a promotion) or `None`.
    pub promotion: Option<Promotion>,
}

impl FromStr for LanMove {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (from_str, rest) = s.split_at_checked(2).ok_or(())?;
        let (to_str, maybe_promotion_str) = rest.split_at_checked(2).ok_or(())?;
        let from = from_str.parse::<Square>()?;
        let to = to_str.parse::<Square>()?;
        let promotion = match maybe_promotion_str {
            "" => None,
            promotion_str => Some(promotion_str.parse::<Promotion>()?),
        };
        Ok(LanMove {
            from,
            to,
            promotion,
        })
    }
}

impl fmt::Display for LanMove {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.from, self.to)?;
        if let Some(promotion) = self.promotion {
            write!(f, "{promotion}")?;
        }
        Ok(())
    }
}

impl Position {
    /// Returns an equivalent `ChessMove` for a `LanMove` in this position.
    pub fn lan_move(&self, lan_move: LanMove) -> Option<ChessMove> {
        let piece = self.board().get_piece_at(lan_move.from)?;
        let mut result = None;
        let mut test_move = |chess_move: ChessMove| {
            if chess_move.lan() == lan_move {
                result = Some(chess_move);
            }
        };
        match piece {
            Piece::Pawn => {
                self.push_pawn_quiets(&mut test_move);
                self.push_pawn_attacks(&mut test_move);
            }
            Piece::Bishop => {
                self.push_bishop_moves(&mut test_move);
            }
            Piece::Knight => {
                self.push_knight_moves(&mut test_move);
            }
            Piece::Rook => {
                self.push_rook_moves(&mut test_move);
            }
            Piece::Queen => {
                self.push_bishop_moves(&mut test_move);
                self.push_rook_moves(&mut test_move);
            }
            Piece::King => {
                self.push_castlings(&mut test_move);
                self.push_king_moves(&mut test_move);
            }
        }

        result
    }

    /// Returns whether a given chess move is at least pseudo-legal in this position.
    #[must_use]
    pub fn is_move_applicable(&self, chess_move: ChessMove) -> bool {
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

    /// Makes a chess move.
    ///
    /// # Preconditions
    /// - `chess_move` must be at least pseduo-legal for this position.
    ///
    /// Violating the preconditions may silently corrupt position state.
    #[must_use]
    pub fn make_move(&mut self, chess_move: ChessMove) -> ChessUnmove {
        // TODO: this check may slow the program down.
        debug_assert!(
            self.is_move_applicable(chess_move),
            "{chess_move:?} is not applicable!"
        );

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
    /// Rolls back a move.
    ///
    /// # Preconditions
    /// - `chess_unmove` must have had been generated from the same move as this position.
    ///
    /// Violating the preconditions may silently corrupt position state.
    pub fn unmake_move(&mut self, chess_unmove: ChessUnmove) {
        self.set_turn(!self.turn());
        self.set_castling_rights(chess_unmove.castling_rights);
        self.set_en_passant(chess_unmove.en_passant);
        self.set_move_index_rule_50(chess_unmove.move_index_rule_50);
        self.set_move_index(self.move_index() - 1);

        let from = chess_unmove.chess_move.from;
        let to = chess_unmove.chess_move.to;
        let hint = chess_unmove.chess_move.hint;

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
                let captured_piece = chess_unmove
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
                let captured_piece = chess_unmove
                    .capture
                    .expect("faulty unmove concept: no captured piece");

                self.remove_color_piece(self.turn(), Piece::Knight, to);
                self.add_color_piece(!self.turn(), captured_piece, to);
                self.add_color_piece(self.turn(), Piece::Pawn, from);
            }
            ChessMoveHint::BishopPromotionCapture => {
                let captured_piece = chess_unmove
                    .capture
                    .expect("faulty unmove concept: no captured piece");

                self.remove_color_piece(self.turn(), Piece::Bishop, to);
                self.add_color_piece(!self.turn(), captured_piece, to);
                self.add_color_piece(self.turn(), Piece::Pawn, from);
            }
            ChessMoveHint::RookPromotionCapture => {
                let captured_piece = chess_unmove
                    .capture
                    .expect("faulty unmove concept: no captured piece");

                self.remove_color_piece(self.turn(), Piece::Rook, to);
                self.add_color_piece(!self.turn(), captured_piece, to);
                self.add_color_piece(self.turn(), Piece::Pawn, from);
            }
            ChessMoveHint::QueenPromotionCapture => {
                let captured_piece = chess_unmove
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
    /// Returns `true` if the kingside castling is disallowed for a given color.
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

    /// Returns `true` if the queenside castling is disallowed for a given color.
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
