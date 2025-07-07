pub mod position;

use crate::game::position::board::bitboard::{BitBoard, File, Square};
use crate::game::position::board::{Color, Piece};
use crate::game::position::{CastlingRights, Position};

#[cfg_attr(test, derive(enum_iterator::Sequence))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MoveHint {
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

impl MoveHint {
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MoveConcept {
    data: u16,
}

impl MoveConcept {
    #[inline(always)]
    fn new(from: Square, to: Square, hint: MoveHint) -> MoveConcept {
        MoveConcept {
            data: (((hint as u16) & 0xf) << 12)
                | (((from as u16) & 0x3f) << 6)
                | ((to as u16) & 0x3f),
        }
    }
    #[inline(always)]
    #[must_use]
    pub fn to(self) -> Square {
        unsafe { std::mem::transmute((self.data & 0x3f) as u8) }
    }
    #[inline(always)]
    #[must_use]
    pub fn from(self) -> Square {
        unsafe { std::mem::transmute(((self.data >> 6) & 0x3f) as u8) }
    }
    #[inline(always)]
    #[must_use]
    pub fn hint(self) -> MoveHint {
        unsafe { std::mem::transmute(((self.data >> 12) & 0x0f) as u8) }
    }
}

impl std::fmt::Debug for MoveConcept {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MoveConcept {{ from: {:?}, to: {:?}, hint: {:?} }}",
            self.from(),
            self.to(),
            self.hint()
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UnmoveConcept {
    move_concept: MoveConcept,
    // Capture does not need to be wrapped, but it's safer.
    // Could maybe remove if it'll be bad for performance.
    capture: Option<Piece>,
    en_passant: Option<File>,
    castling_rights: CastlingRights,
    halfmove_clock: u32,
}

#[derive(Debug, Clone)]
pub struct MoveGenerator {
    moves: Vec<MoveConcept>,
    lens: Vec<usize>,
    len: usize,
}

impl MoveGenerator {
    #[inline(always)]
    #[must_use]
    pub fn empty() -> MoveGenerator {
        MoveGenerator {
            moves: Vec::new(),
            lens: Vec::new(),
            len: 0,
        }
    }
    #[inline(always)]
    fn push_move(&mut self, move_concept: MoveConcept) {
        self.moves.push(move_concept);
        self.len += 1;
    }
    #[inline(always)]
    pub fn pop_move(&mut self) -> Option<MoveConcept> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        self.moves.pop()
    }
    #[inline(always)]
    fn push_group(&mut self) {
        self.lens.push(self.len);
        self.len = 0;
    }
    #[inline(always)]
    pub fn pop_group(&mut self) {
        self.moves.truncate(self.moves.len() - self.len);
        self.len = self.lens.pop().expect("move list has no more groups");
    }
}

impl MoveGenerator {
    fn push_king_moves(&mut self, pos: &Position) {
        let from = pos
            .board()
            .get_color_piece(pos.turn(), Piece::King)
            .bit_scan_forward()
            .expect("king does not exist");
        let attacks = BitBoard::king_attacks(from);

        for to in (pos.board().get_color(!pos.turn()) & attacks).serialize() {
            self.push_move(MoveConcept::new(from, to, MoveHint::Caputre));
        }

        for to in (!pos.board().get_occupance() & attacks).serialize() {
            self.push_move(MoveConcept::new(from, to, MoveHint::Quiet));
        }
    }
    fn push_knight_moves(&mut self, pos: &Position) {
        let opp = pos.board().get_color(!pos.turn());
        let empty = !pos.board().get_occupance();

        for from in pos
            .board()
            .get_color_piece(pos.turn(), Piece::Knight)
            .serialize()
        {
            let attacks = BitBoard::knight_attacks(from);

            for to in (attacks & opp).serialize() {
                self.push_move(MoveConcept::new(from, to, MoveHint::Caputre));
            }

            for to in (attacks & empty).serialize() {
                self.push_move(MoveConcept::new(from, to, MoveHint::Quiet));
            }
        }
    }
    fn push_bishop_moves(&mut self, pos: &Position) {
        let opp = pos.board().get_color(!pos.turn());
        let occ = pos.board().get_occupance();

        for from in pos.board().get_color_bishop_sliders(pos.turn()).serialize() {
            let attacks = BitBoard::bishop_attacks(occ, from);

            for to in (attacks & opp).serialize() {
                self.push_move(MoveConcept::new(from, to, MoveHint::Caputre));
            }

            for to in (attacks & !occ).serialize() {
                self.push_move(MoveConcept::new(from, to, MoveHint::Quiet));
            }
        }
    }
    fn push_rook_moves(&mut self, pos: &Position) {
        let opp = pos.board().get_color(!pos.turn());
        let occ = pos.board().get_occupance();

        for from in pos.board().get_color_rook_sliders(pos.turn()).serialize() {
            let attacks = BitBoard::rook_attacks(occ, from);

            for to in (attacks & opp).serialize() {
                self.push_move(MoveConcept::new(from, to, MoveHint::Caputre));
            }

            for to in (attacks & !occ).serialize() {
                self.push_move(MoveConcept::new(from, to, MoveHint::Quiet));
            }
        }
    }
    fn push_pawn_quiets(&mut self, pos: &Position) {
        let empty = !pos.board().get_occupance();
        let mut single_pushes = BitBoard::pawn_pushes(
            pos.board().get_color_piece(pos.turn(), Piece::Pawn),
            empty,
            pos.turn(),
        );
        let double_pushes = BitBoard::pawn_pushes(
            single_pushes
                & if pos.turn() == Color::White {
                    BitBoard::RANK_3
                } else {
                    BitBoard::RANK_6
                },
            empty,
            pos.turn(),
        );
        let promotion_pushes = single_pushes
            & if pos.turn() == Color::White {
                BitBoard::RANK_8
            } else {
                BitBoard::RANK_1
            };
        single_pushes &= !promotion_pushes;
        let push_offset = if pos.turn() == Color::White { -8 } else { 8 };

        for to in single_pushes.serialize() {
            self.push_move(MoveConcept::new(
                to.shifted(push_offset),
                to,
                MoveHint::Quiet,
            ));
        }

        for to in double_pushes.serialize() {
            self.push_move(MoveConcept::new(
                to.shifted(push_offset * 2),
                to,
                MoveHint::DoublePawn,
            ));
        }

        for to in promotion_pushes.serialize() {
            let from = to.shifted(push_offset);
            self.push_move(MoveConcept::new(from, to, MoveHint::BishopPromotion));
            self.push_move(MoveConcept::new(from, to, MoveHint::KnightPromotion));
            self.push_move(MoveConcept::new(from, to, MoveHint::RookPromotion));
            self.push_move(MoveConcept::new(from, to, MoveHint::QueenPromotion));
        }
    }
    fn push_pawn_attacks(&mut self, pos: &Position) {
        let opp = pos.board().get_color(!pos.turn());

        let mut pawns = pos.board().get_color_piece(pos.turn(), Piece::Pawn);
        let promoters = pawns
            & if pos.turn() == Color::White {
                BitBoard::RANK_7
            } else {
                BitBoard::RANK_2
            };
        pawns &= !promoters;
        for from in promoters.serialize() {
            for to in (BitBoard::pawn_attacks(from, pos.turn()) & opp).serialize() {
                self.push_move(MoveConcept::new(from, to, MoveHint::BishopPromotionCapture));
                self.push_move(MoveConcept::new(from, to, MoveHint::KnightPromotionCapture));
                self.push_move(MoveConcept::new(from, to, MoveHint::RookPromotionCapture));
                self.push_move(MoveConcept::new(from, to, MoveHint::QueenPromotionCapture));
            }
        }

        if pos.en_passant().is_some() {
            let to = Square::new(pos.en_passant().unwrap(), pos.turn().en_passant_dest_rank());
            for from in (pawns & BitBoard::pawn_attacks(to, !pos.turn())).serialize() {
                self.push_move(MoveConcept::new(from, to, MoveHint::EnPassantCapture));
            }
        }

        for from in pawns.serialize() {
            for to in (BitBoard::pawn_attacks(from, pos.turn()) & opp).serialize() {
                self.push_move(MoveConcept::new(from, to, MoveHint::Caputre));
            }
        }
    }
    fn push_castlings(&mut self, pos: &Position) {
        if pos.board().is_king_in_check(pos.turn()) {
            return;
        }

        if !pos.is_kingside_castling_prohibited(pos.turn()) {
            self.push_move(MoveConcept::new(
                if pos.turn() == Color::White {
                    Square::E1
                } else {
                    Square::E8
                },
                if pos.turn() == Color::White {
                    Square::G1
                } else {
                    Square::G8
                },
                MoveHint::KingCastle,
            ));
        }
        if !pos.is_queenside_castling_prohibited(pos.turn()) {
            self.push_move(MoveConcept::new(
                if pos.turn() == Color::White {
                    Square::E1
                } else {
                    Square::E8
                },
                if pos.turn() == Color::White {
                    Square::C1
                } else {
                    Square::C8
                },
                MoveHint::QueenCastle,
            ));
        }
    }
    pub fn generate_moves(&mut self, pos: &Position) {
        // TODO: could optimize for double checks here...
        self.push_group();
        self.push_king_moves(pos);
        self.push_knight_moves(pos);
        self.push_bishop_moves(pos);
        self.push_rook_moves(pos);
        self.push_pawn_attacks(pos);
        self.push_pawn_quiets(pos);
        self.push_castlings(pos);
    }
}

#[must_use]
pub fn can_make_move(pos: &Position, move_concept: MoveConcept) -> bool {
    let from = move_concept.from();
    let to = move_concept.to();
    let hint = move_concept.hint();

    let piece = match pos.board().get_piece_at(from) {
        Some(piece) => piece,
        None => return false,
    };

    let color = match pos.board().get_color_at(from) {
        Some(color) => color,
        None => return false,
    };
    if color != pos.turn() {
        return false;
    }

    let occ = pos.board().get_occupance();
    let empty = pos.board().get_empty();
    let target = pos.board().get_piece_at(to);
    let target_color = pos.board().get_color_at(to);

    if let Some(tgt_color) = target_color {
        if tgt_color == color {
            return false;
        }
    }

    if target.is_some() != hint.is_capture() && hint != MoveHint::EnPassantCapture {
        return false;
    }

    match hint {
        MoveHint::Quiet | MoveHint::Caputre => match piece {
            Piece::Pawn => {
                if piece != Piece::Pawn {
                    return false;
                }
                if hint.is_capture() {
                    !(BitBoard::pawn_attacks(from, color) & BitBoard::from(to)).none()
                } else {
                    !(BitBoard::pawn_pushes(BitBoard::from(from), empty, color)
                        & BitBoard::from(to))
                    .none()
                }
            }
            Piece::Bishop => !(BitBoard::bishop_attacks(occ, from) & BitBoard::from(to)).none(),
            Piece::Knight => !(BitBoard::knight_attacks(from) & BitBoard::from(to)).none(),
            Piece::Rook => !(BitBoard::rook_attacks(occ, from) & BitBoard::from(to)).none(),
            Piece::Queen => !(BitBoard::queen_attacks(occ, from) & BitBoard::from(to)).none(),
            Piece::King => !(BitBoard::king_attacks(from) & BitBoard::from(to)).none(),
        },
        MoveHint::DoublePawn => {
            piece == Piece::Pawn
                && from.into_rank() == color.pawn_rank()
                && BitBoard::pawn_pushes(
                    BitBoard::pawn_pushes(BitBoard::from(from), empty, color),
                    empty,
                    color,
                ) == BitBoard::from(to)
        }
        MoveHint::KingCastle => piece == Piece::King && !pos.is_kingside_castling_prohibited(color),
        MoveHint::QueenCastle => {
            piece == Piece::King && !pos.is_queenside_castling_prohibited(color)
        }
        MoveHint::EnPassantCapture => {
            let file = match pos.en_passant() {
                Some(file) => file,
                None => return false,
            };
            let target_sq = Square::new(to.into_file(), from.into_rank());
            Square::new(file, color.en_passant_dest_rank()) == to
                && !(BitBoard::pawn_attacks(from, color) & BitBoard::from(to)).none()
                && pos.board().get_piece_at(target_sq) == Some(Piece::Pawn)
                && pos.board().get_color_at(target_sq) == Some(!color)
        }
        MoveHint::KnightPromotion
        | MoveHint::BishopPromotion
        | MoveHint::RookPromotion
        | MoveHint::QueenPromotion => {
            piece == Piece::Pawn
                && to.into_rank() == color.promotion_rank()
                && BitBoard::pawn_pushes(BitBoard::from(from), BitBoard::FULL, color)
                    == BitBoard::from(to)
        }
        MoveHint::KnightPromotionCapture
        | MoveHint::BishopPromotionCapture
        | MoveHint::RookPromotionCapture
        | MoveHint::QueenPromotionCapture => {
            piece == Piece::Pawn
                && to.into_rank() == color.promotion_rank()
                && !(BitBoard::pawn_attacks(from, color) & BitBoard::from(to)).none()
        }
    }
}

pub fn make_move(pos: &mut Position, move_concept: MoveConcept) -> UnmoveConcept {
    debug_assert!(
        can_make_move(pos, move_concept),
        "buerak -> {move_concept:?}"
    );
    let from = move_concept.from();
    let to = move_concept.to();
    let hint = move_concept.hint();

    let capture;
    let en_passant = pos.en_passant();
    let castling_rights = pos.castling_rights();
    let halfmove_clock = pos.halfmove_clock();

    match hint {
        MoveHint::Quiet => {
            let piece = pos
                .board()
                .get_piece_at(from)
                .expect("faulty move concept: from is empty");

            capture = None;
            pos.set_en_passant(None);
            pos.set_halfmove_clock(if piece == Piece::Pawn {
                0
            } else {
                pos.halfmove_clock() + 1
            });

            if piece == Piece::King {
                pos.set_castling_rights(
                    pos.castling_rights() & !CastlingRights::both_sides(pos.turn()),
                );
            } else if piece == Piece::Rook {
                if from
                    == (if pos.turn() == Color::White {
                        Square::H1
                    } else {
                        Square::H8
                    })
                {
                    pos.set_castling_rights(
                        pos.castling_rights() & !CastlingRights::kingside(pos.turn()),
                    );
                } else if from
                    == (if pos.turn() == Color::White {
                        Square::A1
                    } else {
                        Square::A8
                    })
                {
                    pos.set_castling_rights(
                        pos.castling_rights() & !CastlingRights::queenside(pos.turn()),
                    );
                }
            }

            pos.remove_color_piece(pos.turn(), piece, from);
            pos.add_color_piece(pos.turn(), piece, to);
        }
        MoveHint::DoublePawn => {
            capture = None;
            pos.set_en_passant(Some(from.into_file()));
            pos.set_halfmove_clock(0);

            pos.remove_color_piece(pos.turn(), Piece::Pawn, from);
            pos.add_color_piece(pos.turn(), Piece::Pawn, to);
        }
        MoveHint::BishopPromotion => {
            capture = None;
            pos.set_en_passant(None);
            pos.set_halfmove_clock(0);

            pos.remove_color_piece(pos.turn(), Piece::Pawn, from);
            pos.add_color_piece(pos.turn(), Piece::Bishop, to);
        }
        MoveHint::KnightPromotion => {
            capture = None;
            pos.set_en_passant(None);
            pos.set_halfmove_clock(0);

            pos.remove_color_piece(pos.turn(), Piece::Pawn, from);
            pos.add_color_piece(pos.turn(), Piece::Knight, to);
        }
        MoveHint::RookPromotion => {
            capture = None;
            pos.set_en_passant(None);
            pos.set_halfmove_clock(0);

            pos.remove_color_piece(pos.turn(), Piece::Pawn, from);
            pos.add_color_piece(pos.turn(), Piece::Rook, to);
        }
        MoveHint::QueenPromotion => {
            capture = None;
            pos.set_en_passant(None);
            pos.set_halfmove_clock(0);

            pos.remove_color_piece(pos.turn(), Piece::Pawn, from);
            pos.add_color_piece(pos.turn(), Piece::Queen, to);
        }
        MoveHint::Caputre => {
            let piece = pos
                .board()
                .get_piece_at(from)
                .expect("faulty move concept: from is empty");
            let captured_piece = pos
                .board()
                .get_piece_at(to)
                .expect("faulty move concept: to is empty");

            capture = Some(captured_piece);
            pos.set_en_passant(None);
            pos.set_halfmove_clock(pos.halfmove_clock() + 1);

            if piece == Piece::King {
                pos.set_castling_rights(
                    pos.castling_rights() & !CastlingRights::both_sides(pos.turn()),
                );
            } else if piece == Piece::Rook {
                if from
                    == (if pos.turn() == Color::White {
                        Square::H1
                    } else {
                        Square::H8
                    })
                {
                    pos.set_castling_rights(
                        pos.castling_rights() & !CastlingRights::kingside(pos.turn()),
                    );
                } else if from
                    == (if pos.turn() == Color::White {
                        Square::A1
                    } else {
                        Square::A8
                    })
                {
                    pos.set_castling_rights(
                        pos.castling_rights() & !CastlingRights::queenside(pos.turn()),
                    );
                }
            }

            pos.remove_color_piece(pos.turn(), piece, from);
            pos.remove_color_piece(!pos.turn(), captured_piece, to);
            pos.add_color_piece(pos.turn(), piece, to);
        }
        MoveHint::EnPassantCapture => {
            capture = Some(Piece::Pawn);
            pos.set_en_passant(None);
            pos.set_halfmove_clock(0);

            pos.remove_color_piece(pos.turn(), Piece::Pawn, from);
            pos.remove_color_piece(
                !pos.turn(),
                Piece::Pawn,
                Square::new(to.into_file(), from.into_rank()),
            );
            pos.add_color_piece(pos.turn(), Piece::Pawn, to);
        }
        MoveHint::BishopPromotionCapture => {
            let captured_piece = pos
                .board()
                .get_piece_at(to)
                .expect("faulty move concept: to is empty");

            capture = Some(captured_piece);
            pos.set_en_passant(None);
            pos.set_halfmove_clock(0);

            pos.remove_color_piece(pos.turn(), Piece::Pawn, from);
            pos.remove_color_piece(!pos.turn(), captured_piece, to);
            pos.add_color_piece(pos.turn(), Piece::Bishop, to);
        }
        MoveHint::KnightPromotionCapture => {
            let captured_piece = pos
                .board()
                .get_piece_at(to)
                .expect("faulty move concept: to is empty");

            capture = Some(captured_piece);
            pos.set_en_passant(None);
            pos.set_halfmove_clock(0);

            pos.remove_color_piece(pos.turn(), Piece::Pawn, from);
            pos.remove_color_piece(!pos.turn(), captured_piece, to);
            pos.add_color_piece(pos.turn(), Piece::Knight, to);
        }
        MoveHint::RookPromotionCapture => {
            let captured_piece = pos
                .board()
                .get_piece_at(to)
                .expect("faulty move concept: to is empty");

            capture = Some(captured_piece);
            pos.set_en_passant(None);
            pos.set_halfmove_clock(0);

            pos.remove_color_piece(pos.turn(), Piece::Pawn, from);
            pos.remove_color_piece(!pos.turn(), captured_piece, to);
            pos.add_color_piece(pos.turn(), Piece::Rook, to);
        }
        MoveHint::QueenPromotionCapture => {
            let captured_piece = pos
                .board()
                .get_piece_at(to)
                .expect("faulty move concept: to is empty");

            capture = Some(captured_piece);
            pos.set_en_passant(None);
            pos.set_halfmove_clock(0);

            pos.remove_color_piece(pos.turn(), Piece::Pawn, from);
            pos.remove_color_piece(!pos.turn(), captured_piece, to);
            pos.add_color_piece(pos.turn(), Piece::Queen, to);
        }
        MoveHint::KingCastle => {
            capture = None;
            pos.set_en_passant(None);
            pos.set_halfmove_clock(pos.halfmove_clock() + 1);
            pos.set_castling_rights(
                pos.castling_rights() & !CastlingRights::both_sides(pos.turn()),
            );

            pos.remove_color_piece(pos.turn(), Piece::King, from);
            pos.add_color_piece(pos.turn(), Piece::King, to);

            pos.remove_color_piece(
                pos.turn(),
                Piece::Rook,
                if pos.turn() == Color::White {
                    Square::H1
                } else {
                    Square::H8
                },
            );
            pos.add_color_piece(
                pos.turn(),
                Piece::Rook,
                if pos.turn() == Color::White {
                    Square::F1
                } else {
                    Square::F8
                },
            );
        }
        MoveHint::QueenCastle => {
            capture = None;
            pos.set_en_passant(None);
            pos.set_halfmove_clock(pos.halfmove_clock() + 1);
            pos.set_castling_rights(
                pos.castling_rights() & !CastlingRights::both_sides(pos.turn()),
            );

            pos.remove_color_piece(pos.turn(), Piece::King, from);
            pos.add_color_piece(pos.turn(), Piece::King, to);

            pos.remove_color_piece(
                pos.turn(),
                Piece::Rook,
                if pos.turn() == Color::White {
                    Square::A1
                } else {
                    Square::A8
                },
            );
            pos.add_color_piece(
                pos.turn(),
                Piece::Rook,
                if pos.turn() == Color::White {
                    Square::D1
                } else {
                    Square::D8
                },
            );
        }
    }

    pos.set_turn(!pos.turn());

    UnmoveConcept {
        move_concept,
        capture,
        en_passant,
        castling_rights,
        halfmove_clock,
    }
}

pub fn unmake_move(board: &mut Position, unmove_concept: UnmoveConcept) {
    board.set_turn(!board.turn());
    board.set_castling_rights(unmove_concept.castling_rights);
    board.set_en_passant(unmove_concept.en_passant);
    board.set_halfmove_clock(unmove_concept.halfmove_clock);

    let from = unmove_concept.move_concept.from();
    let to = unmove_concept.move_concept.to();
    let hint = unmove_concept.move_concept.hint();

    match hint {
        MoveHint::Quiet => {
            let piece = board
                .board()
                .get_piece_at(to)
                .expect("faulty unmove concept: to is empty");

            board.remove_color_piece(board.turn(), piece, to);
            board.add_color_piece(board.turn(), piece, from);
        }
        MoveHint::DoublePawn => {
            board.remove_color_piece(board.turn(), Piece::Pawn, to);
            board.add_color_piece(board.turn(), Piece::Pawn, from);
        }
        MoveHint::KnightPromotion => {
            board.remove_color_piece(board.turn(), Piece::Knight, to);
            board.add_color_piece(board.turn(), Piece::Pawn, from);
        }
        MoveHint::BishopPromotion => {
            board.remove_color_piece(board.turn(), Piece::Bishop, to);
            board.add_color_piece(board.turn(), Piece::Pawn, from);
        }
        MoveHint::RookPromotion => {
            board.remove_color_piece(board.turn(), Piece::Rook, to);
            board.add_color_piece(board.turn(), Piece::Pawn, from);
        }
        MoveHint::QueenPromotion => {
            board.remove_color_piece(board.turn(), Piece::Queen, to);
            board.add_color_piece(board.turn(), Piece::Pawn, from);
        }
        MoveHint::Caputre => {
            let piece = board
                .board()
                .get_piece_at(to)
                .expect("faulty unmove concept: to is empty");
            let captured_piece = unmove_concept
                .capture
                .expect("faulty unmove concept: no captured piece");

            board.remove_color_piece(board.turn(), piece, to);
            board.add_color_piece(!board.turn(), captured_piece, to);
            board.add_color_piece(board.turn(), piece, from);
        }
        MoveHint::EnPassantCapture => {
            board.remove_color_piece(board.turn(), Piece::Pawn, to);
            board.add_color_piece(
                !board.turn(),
                Piece::Pawn,
                Square::new(to.into_file(), from.into_rank()),
            );
            board.add_color_piece(board.turn(), Piece::Pawn, from);
        }
        MoveHint::KnightPromotionCapture => {
            let captured_piece = unmove_concept
                .capture
                .expect("faulty unmove concept: no captured piece");

            board.remove_color_piece(board.turn(), Piece::Knight, to);
            board.add_color_piece(!board.turn(), captured_piece, to);
            board.add_color_piece(board.turn(), Piece::Pawn, from);
        }
        MoveHint::BishopPromotionCapture => {
            let captured_piece = unmove_concept
                .capture
                .expect("faulty unmove concept: no captured piece");

            board.remove_color_piece(board.turn(), Piece::Bishop, to);
            board.add_color_piece(!board.turn(), captured_piece, to);
            board.add_color_piece(board.turn(), Piece::Pawn, from);
        }
        MoveHint::RookPromotionCapture => {
            let captured_piece = unmove_concept
                .capture
                .expect("faulty unmove concept: no captured piece");

            board.remove_color_piece(board.turn(), Piece::Rook, to);
            board.add_color_piece(!board.turn(), captured_piece, to);
            board.add_color_piece(board.turn(), Piece::Pawn, from);
        }
        MoveHint::QueenPromotionCapture => {
            let captured_piece = unmove_concept
                .capture
                .expect("faulty unmove concept: no captured piece");

            board.remove_color_piece(board.turn(), Piece::Queen, to);
            board.add_color_piece(!board.turn(), captured_piece, to);
            board.add_color_piece(board.turn(), Piece::Pawn, from);
        }
        MoveHint::KingCastle => {
            board.remove_color_piece(board.turn(), Piece::King, to);
            board.add_color_piece(board.turn(), Piece::King, from);

            board.remove_color_piece(
                board.turn(),
                Piece::Rook,
                if board.turn() == Color::White {
                    Square::F1
                } else {
                    Square::F8
                },
            );
            board.add_color_piece(
                board.turn(),
                Piece::Rook,
                if board.turn() == Color::White {
                    Square::H1
                } else {
                    Square::H8
                },
            );
        }
        MoveHint::QueenCastle => {
            board.remove_color_piece(board.turn(), Piece::King, to);
            board.add_color_piece(board.turn(), Piece::King, from);

            board.remove_color_piece(
                board.turn(),
                Piece::Rook,
                if board.turn() == Color::White {
                    Square::D1
                } else {
                    Square::D8
                },
            );
            board.add_color_piece(
                board.turn(),
                Piece::Rook,
                if board.turn() == Color::White {
                    Square::A1
                } else {
                    Square::A8
                },
            );
        }
    }
}
