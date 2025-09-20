use crate::{
    game::position::Position, CastlingRights, ChessMove, ChessMoveHint, File, Piece, Square,
};

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
            self.is_move_pseudo_legal(chess_move),
            "{chess_move:?} is not applicable!"
        );

        let from = chess_move.orig_square();
        let to = chess_move.dest_square();
        let hint = chess_move.hint();

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
                    if from == self.turn().mirror_square(Square::H1) {
                        self.set_castling_rights(
                            self.castling_rights() & !CastlingRights::kingside(self.turn()),
                        );
                    } else if from == self.turn().mirror_square(Square::A1) {
                        self.set_castling_rights(
                            self.castling_rights() & !CastlingRights::queenside(self.turn()),
                        );
                    }
                }

                self.move_color_piece(self.turn(), piece, from, to);
            }
            ChessMoveHint::DoublePawn => {
                capture = None;
                self.set_en_passant(Some(from.file()));
                self.set_move_index_rule_50(self.move_index());

                self.move_color_piece(self.turn(), Piece::Pawn, from, to);
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
                    if from == self.turn().mirror_square(Square::H1) {
                        self.set_castling_rights(
                            self.castling_rights() & !CastlingRights::kingside(self.turn()),
                        );
                    } else if from == self.turn().mirror_square(Square::A1) {
                        self.set_castling_rights(
                            self.castling_rights() & !CastlingRights::queenside(self.turn()),
                        );
                    }
                }

                self.remove_color_piece(!self.turn(), captured_piece, to);
                self.move_color_piece(self.turn(), piece, from, to);
            }
            ChessMoveHint::EnPassantCapture => {
                capture = Some(Piece::Pawn);
                self.set_en_passant(None);
                self.set_move_index_rule_50(self.move_index());

                self.remove_color_piece(
                    !self.turn(),
                    Piece::Pawn,
                    Square::new(from.rank(), to.file()),
                );
                self.move_color_piece(self.turn(), Piece::Pawn, from, to);
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

                self.move_color_piece(self.turn(), Piece::King, from, to);
                self.move_color_piece(
                    self.turn(),
                    Piece::Rook,
                    self.turn().mirror_square(Square::H1),
                    self.turn().mirror_square(Square::F1),
                );
            }
            ChessMoveHint::QueenCastle => {
                capture = None;
                self.set_en_passant(None);
                self.set_castling_rights(
                    self.castling_rights() & !CastlingRights::both_sides(self.turn()),
                );

                self.move_color_piece(self.turn(), Piece::King, from, to);
                self.move_color_piece(
                    self.turn(),
                    Piece::Rook,
                    self.turn().mirror_square(Square::A1),
                    self.turn().mirror_square(Square::D1),
                );
            }
        }

        if capture == Some(Piece::Rook) {
            if to == (!self.turn()).mirror_square(Square::H1) {
                self.set_castling_rights(
                    self.castling_rights() & !CastlingRights::kingside(!self.turn()),
                );
            } else if to == (!self.turn()).mirror_square(Square::A1) {
                self.set_castling_rights(
                    self.castling_rights() & !CastlingRights::queenside(!self.turn()),
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

        let from = chess_unmove.chess_move.orig_square();
        let to = chess_unmove.chess_move.dest_square();
        let hint = chess_unmove.chess_move.hint();

        match hint {
            ChessMoveHint::Quiet => {
                let piece = self
                    .board()
                    .get_piece_at(to)
                    .expect("faulty unmove concept: to is empty");

                self.move_color_piece(self.turn(), piece, to, from);
            }
            ChessMoveHint::DoublePawn => {
                self.move_color_piece(self.turn(), Piece::Pawn, to, from);
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

                self.move_color_piece(self.turn(), piece, to, from);
                self.add_color_piece(!self.turn(), captured_piece, to);
            }
            ChessMoveHint::EnPassantCapture => {
                self.move_color_piece(self.turn(), Piece::Pawn, to, from);
                self.add_color_piece(
                    !self.turn(),
                    Piece::Pawn,
                    Square::new(from.rank(), to.file()),
                );
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
                self.move_color_piece(self.turn(), Piece::King, to, from);
                self.move_color_piece(
                    self.turn(),
                    Piece::Rook,
                    self.turn().mirror_square(Square::F1),
                    self.turn().mirror_square(Square::H1),
                );
            }
            ChessMoveHint::QueenCastle => {
                self.move_color_piece(self.turn(), Piece::King, to, from);
                self.move_color_piece(
                    self.turn(),
                    Piece::Rook,
                    self.turn().mirror_square(Square::D1),
                    self.turn().mirror_square(Square::A1),
                );
            }
        }
    }
}
