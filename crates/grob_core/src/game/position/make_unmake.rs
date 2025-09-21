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
    halfmove_clock: u32,
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

        let en_passant = self.en_passant();
        let castling_rights = self.castling_rights();

        let piece = self.board().get_piece_at(from).unwrap();
        let capture = self.board().get_piece_at(to);

        match hint {
            ChessMoveHint::Quiet => {
                self.move_color_piece(self.turn(), piece, from, to);
            }
            ChessMoveHint::DoublePawn => {
                self.move_color_piece(self.turn(), Piece::Pawn, from, to);
            }
            ChessMoveHint::BishopPromotion => {
                self.remove_color_piece(self.turn(), Piece::Pawn, from);
                self.add_color_piece(self.turn(), Piece::Bishop, to);
            }
            ChessMoveHint::KnightPromotion => {
                self.remove_color_piece(self.turn(), Piece::Pawn, from);
                self.add_color_piece(self.turn(), Piece::Knight, to);
            }
            ChessMoveHint::RookPromotion => {
                self.remove_color_piece(self.turn(), Piece::Pawn, from);
                self.add_color_piece(self.turn(), Piece::Rook, to);
            }
            ChessMoveHint::QueenPromotion => {
                self.remove_color_piece(self.turn(), Piece::Pawn, from);
                self.add_color_piece(self.turn(), Piece::Queen, to);
            }
            ChessMoveHint::Caputre => {
                self.remove_color_piece(!self.turn(), capture.unwrap(), to);
                self.move_color_piece(self.turn(), piece, from, to);
            }
            ChessMoveHint::EnPassantCapture => {
                let attacked_sq = Square::new(from.rank(), to.file());
                self.remove_color_piece(!self.turn(), Piece::Pawn, attacked_sq);
                self.move_color_piece(self.turn(), Piece::Pawn, from, to);
            }
            ChessMoveHint::BishopPromotionCapture => {
                self.remove_color_piece(self.turn(), Piece::Pawn, from);
                self.remove_color_piece(!self.turn(), capture.unwrap(), to);
                self.add_color_piece(self.turn(), Piece::Bishop, to);
            }
            ChessMoveHint::KnightPromotionCapture => {
                self.remove_color_piece(self.turn(), Piece::Pawn, from);
                self.remove_color_piece(!self.turn(), capture.unwrap(), to);
                self.add_color_piece(self.turn(), Piece::Knight, to);
            }
            ChessMoveHint::RookPromotionCapture => {
                self.remove_color_piece(self.turn(), Piece::Pawn, from);
                self.remove_color_piece(!self.turn(), capture.unwrap(), to);
                self.add_color_piece(self.turn(), Piece::Rook, to);
            }
            ChessMoveHint::QueenPromotionCapture => {
                self.remove_color_piece(self.turn(), Piece::Pawn, from);
                self.remove_color_piece(!self.turn(), capture.unwrap(), to);
                self.add_color_piece(self.turn(), Piece::Queen, to);
            }
            ChessMoveHint::KingCastle => {
                let rook_to = self.turn().mirror_square(Square::F1);
                let rook_from = self.turn().mirror_square(Square::H1);
                self.move_color_piece(self.turn(), Piece::King, from, to);
                self.move_color_piece(self.turn(), Piece::Rook, rook_from, rook_to);
            }
            ChessMoveHint::QueenCastle => {
                self.move_color_piece(self.turn(), Piece::King, from, to);
                self.move_color_piece(
                    self.turn(),
                    Piece::Rook,
                    self.turn().mirror_square(Square::A1),
                    self.turn().mirror_square(Square::D1),
                );
            }
        }

        let reset_hm_clock = match hint {
            ChessMoveHint::Quiet => matches!(piece, Piece::Pawn),
            ChessMoveHint::DoublePawn => true,
            _ if hint.is_capture() || hint.is_promotion() => true,
            _ => false,
        };

        let remove_castling_rights = {
            let from_castling_rights = match hint {
                ChessMoveHint::KingCastle | ChessMoveHint::QueenCastle => {
                    CastlingRights::both_sides(self.turn())
                }
                ChessMoveHint::Quiet | ChessMoveHint::Caputre => match piece {
                    Piece::King => CastlingRights::both_sides(self.turn()),
                    Piece::Rook => {
                        if from == self.turn().mirror_square(Square::H1) {
                            CastlingRights::kingside(self.turn())
                        } else if from == self.turn().mirror_square(Square::A1) {
                            CastlingRights::queenside(self.turn())
                        } else {
                            CastlingRights::empty()
                        }
                    }
                    _ => CastlingRights::empty(),
                },
                _ => CastlingRights::empty(),
            };
            let to_castling_rights = match capture {
                Some(Piece::Rook) => {
                    // If the rook is captured the player can no longer use it to castle.
                    if to == (!self.turn()).mirror_square(Square::H1) {
                        CastlingRights::kingside(!self.turn())
                    } else if to == (!self.turn()).mirror_square(Square::A1) {
                        CastlingRights::queenside(!self.turn())
                    } else {
                        CastlingRights::empty()
                    }
                }
                _ => CastlingRights::empty(),
            };
            from_castling_rights | to_castling_rights
        };
        self.set_castling_rights(self.castling_rights() & !remove_castling_rights);

        self.set_en_passant(match hint {
            ChessMoveHint::DoublePawn => Some(from.file()),
            _ => None,
        });

        self.swap_turn();

        let halfmove_clock = self.next_move_index(reset_hm_clock);

        ChessUnmove {
            chess_move,
            capture,
            en_passant,
            castling_rights,
            halfmove_clock,
        }
    }
    /// Rolls back a move.
    ///
    /// # Preconditions
    /// - `chess_unmove` must have had been generated from the same move as this position.
    ///
    /// Violating the preconditions may silently corrupt position state.
    pub fn unmake_move(&mut self, chess_unmove: ChessUnmove) {
        self.swap_turn();
        self.set_castling_rights(chess_unmove.castling_rights);
        self.set_en_passant(chess_unmove.en_passant);
        self.prev_move_index(chess_unmove.halfmove_clock);

        let from = chess_unmove.chess_move.orig_square();
        let to = chess_unmove.chess_move.dest_square();
        let hint = chess_unmove.chess_move.hint();

        let piece = self.board().get_piece_at(to).unwrap();
        let capture = chess_unmove.capture;

        match hint {
            ChessMoveHint::Quiet => {
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
                self.move_color_piece(self.turn(), piece, to, from);
                self.add_color_piece(!self.turn(), capture.unwrap(), to);
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
                self.remove_color_piece(self.turn(), Piece::Knight, to);
                self.add_color_piece(!self.turn(), capture.unwrap(), to);
                self.add_color_piece(self.turn(), Piece::Pawn, from);
            }
            ChessMoveHint::BishopPromotionCapture => {
                self.remove_color_piece(self.turn(), Piece::Bishop, to);
                self.add_color_piece(!self.turn(), capture.unwrap(), to);
                self.add_color_piece(self.turn(), Piece::Pawn, from);
            }
            ChessMoveHint::RookPromotionCapture => {
                self.remove_color_piece(self.turn(), Piece::Rook, to);
                self.add_color_piece(!self.turn(), capture.unwrap(), to);
                self.add_color_piece(self.turn(), Piece::Pawn, from);
            }
            ChessMoveHint::QueenPromotionCapture => {
                self.remove_color_piece(self.turn(), Piece::Queen, to);
                self.add_color_piece(!self.turn(), capture.unwrap(), to);
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
