use std::num::NonZeroU64;

use crate::{
    bitboard::BitBoard,
    board::Board,
    castling_rights::CastlingRights,
    pieces::{Color, Piece},
    square::{File, Rank, Square},
};

mod zobrist;

/// A valid chess position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    board: Board,
    turn: Color,
    castling: CastlingRights,
    en_passant: Option<File>,
    zobrist: u64,
}

impl Position {
    /// Constructs the chess position from its parts.
    ///
    /// The result must be a vaild chess position:
    /// - There must be exactly one king per each color.
    /// - The position should not allow capturing the opponent's king.
    /// - If casling rights are specified, the corresponding
    ///   castlings should be logically possible on the specified board.
    /// - If the en passant is sepcified, the board should be allow to perfrom it.
    pub fn from_parts(
        board: Board,
        turn: Color,
        castling: CastlingRights,
        en_passant: Option<File>,
    ) -> Option<Self> {
        if board.get_color_piece(Color::White, Piece::King).count() != 1
            || board.get_color_piece(Color::Black, Piece::King).count() != 1
            || board.is_king_in_check(!turn)
            || !is_castling_allowed(board, castling)
            || !en_passant.is_none_or(|file| is_en_passant_allowed(board, turn, file))
        {
            return None;
        }
        let zobrist = {
            let mut hash = 0;
            for sq in board.get_occupied() {
                let (color, piece) = board.get_color_piece_at(sq).unwrap();
                hash ^= zobrist::get_square_zobrist(color, piece, sq);
            }
            hash ^= zobrist::get_turn_zobrist(turn);
            hash ^= zobrist::get_castling_zobrist(castling);
            hash ^= zobrist::get_en_passant_zobrist(en_passant);
            hash
        };
        Some(Self {
            board,
            turn,
            castling,
            en_passant,
            zobrist,
        })
    }
    /// Returns the zobrist hash for the current position.
    #[must_use]
    pub fn zobrist(&self) -> NonZeroU64 {
        NonZeroU64::new(self.zobrist).unwrap_or(NonZeroU64::MAX)
    }
    /// Returns the file of the available en passant.
    #[must_use]
    pub fn en_passant(&self) -> Option<File> {
        self.en_passant
    }
    /// Returns the castling rights for this position.
    #[must_use]
    pub fn castling_rights(&self) -> CastlingRights {
        self.castling
    }
    /// Returns the board state.
    #[must_use]
    pub fn board(&self) -> &Board {
        &self.board
    }
    /// Returns the color of the player making the trun.
    #[must_use]
    pub fn turn(&self) -> Color {
        self.turn
    }
    /// Sets (or resets) the en passant file.
    ///
    /// #Panics
    /// In debug mode panics when trying to set en passant
    /// that is not possible considering the state of the board.
    pub fn set_en_passant(&mut self, en_passant: Option<File>) {
        if let Some(en_passant_file) = en_passant {
            debug_assert!(is_en_passant_allowed(
                self.board,
                self.turn,
                en_passant_file
            ));
        }
        self.zobrist ^= zobrist::get_en_passant_zobrist(self.en_passant);
        self.zobrist ^= zobrist::get_en_passant_zobrist(en_passant);
        self.en_passant = en_passant;
    }
    /// Sets the castling rights.
    ///
    /// #Panics
    /// In debug mode panics when trying to set casling rights
    /// that are logically impossible for this position.
    pub fn set_castling_rights(&mut self, castling_rights: CastlingRights) {
        debug_assert!(is_castling_allowed(self.board, self.castling));
        self.zobrist ^= zobrist::get_castling_zobrist(self.castling);
        self.zobrist ^= zobrist::get_castling_zobrist(castling_rights);
        self.castling = castling_rights;
    }
    /// Adds a piece to the board.
    ///
    /// This will update the zobrist hash.
    ///
    /// # Panics
    /// If trying to add the piece to an already occupied square
    #[inline(always)]
    pub fn add_color_piece(&mut self, color: Color, piece: Piece, sq: Square) {
        debug_assert!(!self.board.get_occupied().has_square(sq));
        self.zobrist ^= zobrist::get_square_zobrist(color, piece, sq);
        self.board.mask_or(color, piece, BitBoard::from(sq));
    }
    /// Removes a piece from the board.
    ///
    /// This will update the zobrist hash.
    ///
    /// # Panics
    /// - if trying to remove an unoccupied square
    /// - if the `sq` contains the piece of different type or color than specified.
    #[inline(always)]
    pub fn remove_color_piece(&mut self, color: Color, piece: Piece, sq: Square) {
        debug_assert!(
            self.board.get_piece(piece).has_square(sq)
                && self.board.get_color(color).has_square(sq)
        );
        self.zobrist ^= zobrist::get_square_zobrist(color, piece, sq);
        self.board.mask_and(color, piece, !BitBoard::from(sq));
    }
    /// Moves a piece on the board.
    ///
    /// This will update the zobrist hash.
    ///
    /// # Panics
    /// - if trying to move a piece from an unoccupied square
    /// - if `from` contains a piece with a different type or color than specified
    /// - if `to` is occupied
    #[inline(always)]
    pub fn move_color_piece(&mut self, color: Color, piece: Piece, from: Square, to: Square) {
        debug_assert!(
            self.board.get_piece(piece).has_square(from)
                && self.board.get_color(color).has_square(from)
        );
        debug_assert!(!self.board.get_occupied().has_square(to));
        self.zobrist ^= zobrist::get_square_zobrist(color, piece, from);
        self.zobrist ^= zobrist::get_square_zobrist(color, piece, to);
        self.board
            .mask_xor(color, piece, BitBoard::from(from) | BitBoard::from(to));
    }
    /// Changes the color of side to move.
    ///
    /// This will update the zobrist hash.
    #[inline(always)]
    pub fn swap_turn(&mut self) {
        self.zobrist ^= zobrist::get_turn_zobrist(self.turn);
        self.turn = !self.turn;
        self.zobrist ^= zobrist::get_turn_zobrist(self.turn);
    }
}

/// Returns `true` if the position can have the specified castling rights
/// considering the current state of the board.
fn is_castling_allowed(board: Board, castling: CastlingRights) -> bool {
    let castling_max = {
        let mut cr = CastlingRights::all();
        let w_king = board.get_color_piece(Color::White, Piece::King);
        if !w_king.has_square(Square::E1) {
            cr &= !CastlingRights::both_sides(Color::White);
        } else {
            let w_rooks = board.get_color_piece(Color::White, Piece::Rook);
            if !w_rooks.has_square(Square::H1) {
                cr &= !CastlingRights::WHITE_KING;
            }
            if !w_rooks.has_square(Square::A1) {
                cr &= !CastlingRights::WHITE_QUEEN;
            }
        }
        let b_king = board.get_color_piece(Color::Black, Piece::King);
        if !b_king.has_square(Square::E8) {
            cr &= !CastlingRights::both_sides(Color::Black);
        } else {
            let b_rooks = board.get_color_piece(Color::Black, Piece::Rook);
            if !b_rooks.has_square(Square::H8) {
                cr &= !CastlingRights::BLACK_KING;
            }
            if !b_rooks.has_square(Square::A8) {
                cr &= !CastlingRights::BLACK_QUEEN;
            }
        }
        cr
    };
    castling_max.contains(castling)
}

/// Returns `true` if it's logically possible to perform en passant at the specified file
/// on the specified board.
fn is_en_passant_allowed(board: Board, turn: Color, en_passant_file: File) -> bool {
    let sq = Square::new(turn.mirror_rank(Rank::R5), en_passant_file);
    let adj = {
        let bb = BitBoard::from(sq);
        bb.left() | bb.right()
    };
    board.get_color_piece_at(sq) == Some((!turn, Piece::Pawn))
        && (board.get_color_piece(turn, Piece::Pawn) & adj) != BitBoard::EMPTY
}
