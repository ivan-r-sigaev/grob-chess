use std::{collections::VecDeque, error::Error, fmt, num::NonZeroU64};

use crate::{
    game::{
        make::ChessUnmove,
        zobrist::{
            get_castling_zobrist, get_en_passant_zobrist, get_square_zobrist, get_turn_zobrist,
        },
    },
    BitBoard, Board, CastlingRights, Color, File, Piece, Rank, Square,
};

/// An error that originated from [FEN] parsing.
///
/// [fen]: https://www.chessprogramming.org/Forsyth-Edwards_Notation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParseFenError {
    /// Something is wrong with the part of the FEN representing the board.
    BadBoard,
    /// Something is wrong with the part of the FEN representing the turn.
    BadTurn,
    /// Something is wrong with the part of the FEN representing the en passant.
    BadEnPassant,
    /// Something is wrong with the part of the FEN representing the castling rights.
    BadCastlingRights,
    /// Something is wrong with the part of the FEN representing the halfmove clock.
    BadHalfmoveClock,
    /// Something is wrong with the part of the FEN representing the fullmove clock.
    BadFullmoveClock,
    /// FEN is valid but the string is followed by an unknown extension.
    TrailingGarbage,
}

impl fmt::Display for ParseFenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Error for ParseFenError {}

/// A chess position.
#[derive(Debug, Clone)]
pub struct Game {
    board: Board,
    history: Vec<PlyHistory>,
    zobrist_hash: u64,
    move_index_rule_50: u32,
    move_index: u32,
    turn: Color,
    en_passant: Option<File>,
    castling_rights: CastlingRights,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct PlyHistory {
    pub(super) hash: NonZeroU64,
    pub(super) unmove: ChessUnmove,
}

impl Game {
    /// Returns the initial position for the standard chess game.
    pub fn initial_position() -> Self {
        const INITIAL_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        Self::try_from_fen(INITIAL_FEN).unwrap()
    }
    /// Tries to parse a positioin from FEN.
    pub fn try_from_fen(fen: &str) -> Result<Self, ParseFenError> {
        let mut words: VecDeque<&str> = fen.split_whitespace().collect();
        let mut zobrist_hash = 0;

        let board = {
            let fen = words.pop_front().ok_or(ParseFenError::BadBoard)?;
            let rows: Vec<&str> = fen.split('/').collect();
            if rows.len() != 8 {
                return Err(ParseFenError::BadBoard);
            }

            let mut board = Board::empty();
            let mut sq: Square = Square::A1;
            for y in (0..8).rev() {
                let mut row_len = 0;
                for ch in rows[y].chars() {
                    if matches!(ch, '1'..='8') {
                        let inc = ch.to_digit(10).unwrap() - 1;
                        row_len += inc;
                        sq = sq.shifted(inc as i8);
                    } else {
                        let piece = ch
                            .to_string()
                            .parse::<Piece>()
                            .map_err(|_| ParseFenError::BadBoard)?;

                        let color = match ch.is_ascii_lowercase() {
                            true => Color::Black,
                            false => Color::White,
                        };

                        board.mask_or(color, piece, BitBoard::from(sq));
                        zobrist_hash ^= get_square_zobrist(color, piece, sq);
                    }
                    sq = sq.shifted(1);
                    row_len += 1;
                    if row_len > 8 {
                        return Err(ParseFenError::BadBoard);
                    }
                }
                if row_len < 8 {
                    return Err(ParseFenError::BadBoard);
                }
            }
            if board.get_color_piece(Color::White, Piece::King).count() != 1 {
                return Err(ParseFenError::BadBoard);
            }
            if board.get_color_piece(Color::Black, Piece::King).count() != 1 {
                return Err(ParseFenError::BadBoard);
            }

            board
        };

        let turn = {
            let turn = words
                .pop_front()
                .and_then(|s| s.parse::<Color>().ok())
                .ok_or(ParseFenError::BadTurn)?;
            zobrist_hash ^= get_turn_zobrist(turn);

            if board.is_king_in_check(!turn) {
                return Err(ParseFenError::BadBoard);
            }
            turn
        };

        let castling_rights = {
            let castling_rights = words
                .pop_front()
                .and_then(|s| s.parse::<CastlingRights>().ok())
                .ok_or(ParseFenError::BadTurn)?;
            let castling_rights_max = {
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
            if !castling_rights_max.contains(castling_rights) {
                return Err(ParseFenError::BadCastlingRights);
            }
            zobrist_hash ^= get_castling_zobrist(castling_rights);
            castling_rights
        };

        let en_passant = {
            let fen = words.pop_front().ok_or(ParseFenError::BadEnPassant)?;
            let en_passant = match fen {
                "-" => None,
                s => {
                    let file = s.parse::<File>().map_err(|_| ParseFenError::BadEnPassant)?;
                    let sq = Square::new(turn.mirror_rank(Rank::R5), file);
                    if board.get_piece_at(sq) != Some(Piece::Pawn)
                        || board.get_color_at(sq) != Some(!turn)
                    {
                        return Err(ParseFenError::BadEnPassant);
                    }
                    Some(file)
                }
            };
            zobrist_hash ^= get_en_passant_zobrist(en_passant);
            en_passant
        };

        let hm = words
            .pop_front()
            .and_then(|s| s.parse::<u32>().ok())
            .ok_or(ParseFenError::BadHalfmoveClock)?;

        let fm = words
            .pop_front()
            .and_then(|s| s.parse::<u32>().ok())
            .ok_or(ParseFenError::BadFullmoveClock)?;

        if fm == 0 {
            return Err(ParseFenError::BadFullmoveClock);
        }

        let move_index = (fm - 1) * 2 + (turn == Color::Black) as u32;

        if hm > move_index {
            return Err(ParseFenError::BadHalfmoveClock);
        }

        let move_index_rule_50 = move_index - hm;

        if !words.is_empty() {
            return Err(ParseFenError::TrailingGarbage);
        }

        let history = Vec::new();

        Ok(Game {
            board,
            turn,
            castling_rights,
            en_passant,
            move_index,
            move_index_rule_50,
            zobrist_hash,
            history,
        })
    }
    /// Returns a hash for the current position.
    #[must_use]
    pub fn zobrist(&self) -> NonZeroU64 {
        NonZeroU64::new(self.zobrist_hash).unwrap_or(NonZeroU64::MAX)
    }
    /// Returns the possible en passant target file if available or `None`.
    #[must_use]
    pub fn en_passant(&self) -> Option<File> {
        self.en_passant
    }
    /// Returns the state of castling rights.
    #[must_use]
    pub fn castling_rights(&self) -> CastlingRights {
        self.castling_rights
    }
    /// Returns a reference to the position's board.
    #[must_use]
    pub fn board(&self) -> &Board {
        &self.board
    }
    /// Returns the color of the player who is about to make a turn.
    #[must_use]
    pub fn turn(&self) -> Color {
        self.turn
    }
    /// Returns the current state of the halfmove clock.
    #[must_use]
    pub fn halfmove_clock(&self) -> u32 {
        self.move_index - self.move_index_rule_50
    }
    /// Returns the number of plies played so far.
    #[must_use]
    pub fn ply_index(&self) -> u32 {
        self.move_index
    }
    /// Returns the number of times this position was played before in the game.
    pub fn count_repetitions(&self) -> usize {
        let hash = self.zobrist();
        self.history.iter().filter(|&ply| ply.hash == hash).count()
    }
    /// Returns `true` if there are no moves to roll back.
    pub fn is_history_empty(&self) -> bool {
        self.history.is_empty()
    }
    /// Returns `true` if the king of the playing player is currently in check.
    pub fn is_check(&self) -> bool {
        self.board().is_king_in_check(self.turn())
    }
    /// Returns `true` if the king of the opponent player is currently in check.
    pub(super) fn was_check_ignored(&self) -> bool {
        self.board().is_king_in_check(!self.turn())
    }
    /// Sets the currently available en passant file.
    ///
    /// This will update the zobrist hash.
    pub(super) fn set_en_passant(&mut self, en_passant: Option<File>) {
        self.zobrist_hash ^= get_en_passant_zobrist(self.en_passant);
        self.zobrist_hash ^= get_en_passant_zobrist(en_passant);
        self.en_passant = en_passant;
    }
    /// Sets the state of castling rights.
    ///
    /// This will update the zobrist hash.
    pub(super) fn set_castling_rights(&mut self, castling_rights: CastlingRights) {
        self.zobrist_hash ^= get_castling_zobrist(self.castling_rights);
        self.zobrist_hash ^= get_castling_zobrist(castling_rights);
        self.castling_rights = castling_rights;
    }
    /// Adds a piece to the board.
    ///
    /// This will update the zobrist hash.
    ///
    /// # Panics
    /// If trying to add the piece to an already occupied square
    #[inline(always)]
    pub(super) fn add_color_piece(&mut self, color: Color, piece: Piece, sq: Square) {
        debug_assert!(!self.board.get_occupance().has_square(sq));
        self.zobrist_hash ^= get_square_zobrist(color, piece, sq);
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
    pub(super) fn remove_color_piece(&mut self, color: Color, piece: Piece, sq: Square) {
        debug_assert!(
            self.board.get_piece(piece).has_square(sq)
                && self.board.get_color(color).has_square(sq)
        );
        self.zobrist_hash ^= get_square_zobrist(color, piece, sq);
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
    pub(super) fn move_color_piece(
        &mut self,
        color: Color,
        piece: Piece,
        from: Square,
        to: Square,
    ) {
        debug_assert!(
            self.board.get_piece(piece).has_square(from)
                && self.board.get_color(color).has_square(from)
        );
        debug_assert!(!self.board.get_occupance().has_square(to));
        self.zobrist_hash ^= get_square_zobrist(color, piece, from);
        self.zobrist_hash ^= get_square_zobrist(color, piece, to);
        self.board
            .mask_xor(color, piece, BitBoard::from(from) | BitBoard::from(to));
    }
    /// Changes the color of side to move.
    ///
    /// This will update the zobrist hash.
    #[inline(always)]
    pub(super) fn swap_turn(&mut self) {
        self.zobrist_hash ^= get_turn_zobrist(self.turn);
        self.turn = !self.turn;
        self.zobrist_hash ^= get_turn_zobrist(self.turn);
    }
    /// Incremetns the move index and returns the previous state of halfmove clock.
    pub(super) fn next_move_index(&mut self, reset_hm_clock: bool) -> u32 {
        let res = self.move_index - self.move_index_rule_50;
        self.move_index += 1;
        if reset_hm_clock {
            self.move_index_rule_50 = self.move_index;
        }
        res
    }
    /// Decremetns the move index and set the state of halfmove clock.
    pub(super) fn prev_move_index(&mut self, hm_clock_state: u32) {
        self.move_index = self.move_index.strict_sub(1);
        self.move_index_rule_50 = self.move_index.strict_sub(hm_clock_state);
    }
    /// Pushes the ply data onto the history stack.
    pub(super) fn push_history(&mut self, history: PlyHistory) {
        self.history.push(history);
    }
    /// Pops the ply data from the history stack.
    pub(super) fn pop_history(&mut self) -> Option<PlyHistory> {
        self.history.pop()
    }
}
