use crate::ChessUnmove;
use std::collections::VecDeque;
use std::error::Error;
use std::fmt::{self, Display};
use std::hash::Hash;
use std::num::NonZeroU64;

use board::{BitBoard, Board, Color, File, Piece, Square};

use crate::zobrist::{
    get_castling_zobrist, get_en_passant_zobrist, get_square_zobrist, get_turn_zobrist,
};
use crate::CastlingRights;

/// An error that originated from [FEN] parsing.
///
/// If a FEN string contains multiple errors the error in the part
/// nearest to the string beginning will be retruned (e.g. FEN with
/// erroneous castling rights and trailing garbage will return
/// [`ParseFenError::BadCastlingRights`]).
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

impl Display for ParseFenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Error for ParseFenError {}

/// A chess game.
#[derive(Debug, Clone)]
pub struct Game {
    board: Board,
    turn: Color,
    castling_rights: CastlingRights,
    en_passant: Option<File>,
    move_index_rule_50: u32,
    move_index: u32,
    zobrist_hash: u64,
    history: Vec<PlyHistory>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct PlyHistory {
    pub hash: NonZeroU64,
    pub unmove: ChessUnmove,
}

impl Game {
    /// Returns the initial position for a normal chess game.
    pub fn initial_position() -> Self {
        const INITIAL_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        Self::try_from_fen(INITIAL_FEN).unwrap()
    }

    /// Tries to parse a new game from [FEN].
    ///
    /// # Examples
    /// ```rust
    /// use position::{Position, ParseFenError};
    ///
    /// const INITIAL_POSITION_FEN: &str =
    ///     "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    /// let initial_position_from_fen = Position::try_from_fen(INITIAL_POSITION_FEN);
    /// assert_eq!(initial_position_from_fen, Ok(Position::initial_position()))
    /// ```
    ///
    /// [fen]: https://www.chessprogramming.org/Forsyth-Edwards_Notation
    pub fn try_from_fen(fen: &str) -> Result<Self, ParseFenError> {
        let mut words: VecDeque<&str> = fen.split_whitespace().collect();
        let mut zobrist_hash = 0;

        let fen_board = words.pop_front().ok_or(ParseFenError::BadBoard)?;
        let rows: Vec<&str> = fen_board.split('/').collect();
        if rows.len() != 8 {
            return Err(ParseFenError::BadBoard);
        }
        let mut board = Board::empty();
        let mut sq: Square = Square::A1;
        for y in (0..8).rev() {
            let mut row_size = 0;
            for ch in rows[y].chars() {
                if ('1'..='8').contains(&ch) {
                    let inc = u8::try_from(ch).unwrap() - u8::try_from('1').unwrap();
                    row_size += inc;
                    sq = sq.shifted(inc as i8);
                } else {
                    let color = match ch.is_ascii_lowercase() {
                        true => Color::Black,
                        false => Color::White,
                    };

                    let piece = ch
                        .to_string()
                        .parse::<Piece>()
                        .map_err(|_| ParseFenError::BadBoard)?;

                    board.mask_or(color, piece, BitBoard::from(sq));
                    zobrist_hash ^= get_square_zobrist(color, piece, sq);
                }
                sq = sq.shifted(1);
                row_size += 1;
                if row_size > 8 {
                    return Err(ParseFenError::BadBoard);
                }
            }
            if row_size < 8 {
                return Err(ParseFenError::BadBoard);
            }
        }
        if board.get_color_piece(Color::White, Piece::King).count() != 1 {
            return Err(ParseFenError::BadBoard);
        }
        if board.get_color_piece(Color::Black, Piece::King).count() != 1 {
            return Err(ParseFenError::BadBoard);
        }

        let turn = words
            .pop_front()
            .and_then(|s| s.parse::<Color>().ok())
            .ok_or(ParseFenError::BadTurn)?;
        zobrist_hash ^= get_turn_zobrist(turn);

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

        let en_passant_fen = words.pop_front().ok_or(ParseFenError::BadEnPassant)?;
        let en_passant = match en_passant_fen {
            "-" => None,
            s => Some(s.parse::<File>().map_err(|_| ParseFenError::BadEnPassant)?),
        };
        zobrist_hash ^= get_en_passant_zobrist(en_passant);

        let hm = words
            .pop_front()
            .and_then(|s| s.parse::<u32>().ok())
            .ok_or(ParseFenError::BadHalfmoveClock)?;

        let fm = words
            .pop_front()
            .and_then(|s| s.parse::<u32>().ok())
            .ok_or(ParseFenError::BadFullmoveClock)?;

        if !words.is_empty() {
            return Err(ParseFenError::TrailingGarbage);
        }

        Ok(Game {
            board,
            turn,
            castling_rights,
            en_passant,
            move_index: fm,
            move_index_rule_50: fm - hm,
            zobrist_hash,
            history: Vec::new(),
        })
    }
    /// Returns a hash for the current position.
    #[inline(always)]
    #[must_use]
    pub fn zobrist(&self) -> NonZeroU64 {
        NonZeroU64::new(self.zobrist_hash).unwrap_or(NonZeroU64::MAX)
    }
    /// Returns the possible en passant target file if available or `None`.
    #[inline(always)]
    #[must_use]
    pub fn en_passant(&self) -> Option<File> {
        self.en_passant
    }
    /// Returns the state of castling rights.
    #[inline(always)]
    #[must_use]
    pub fn castling_rights(&self) -> CastlingRights {
        self.castling_rights
    }
    /// Returns a reference to the position's board.
    #[inline(always)]
    #[must_use]
    pub fn board(&self) -> &Board {
        &self.board
    }
    /// Returns the color of the player who is about to make a turn.
    #[inline(always)]
    #[must_use]
    pub fn turn(&self) -> Color {
        self.turn
    }
    /// Returns the move index when the 50 move rule counter was last reset.
    #[inline(always)]
    #[must_use]
    pub fn move_index_rule_50(&self) -> u32 {
        self.move_index_rule_50
    }
    /// Returns the current move index.
    #[inline(always)]
    #[must_use]
    pub fn move_index(&self) -> u32 {
        self.move_index
    }
    /// Returns whether the king of the playing player is currently in check.
    pub fn is_check(&self) -> bool {
        self.board.is_king_in_check(self.turn())
    }
    /// Returns whether the king of the opponent player is currently in check.
    pub fn was_check_ignored(&self) -> bool {
        self.board.is_king_in_check(!self.turn())
    }
    /// Returns the number of times this position was played before in the game.
    pub fn count_repetitions(&self) -> usize {
        let hash = self.zobrist();
        self.history.iter().filter(|&ply| ply.hash == hash).count()
    }
    /// Returns `true` if this is the starting position for the game.
    pub fn is_history_empty(&self) -> bool {
        self.history.is_empty()
    }
    // Sets the currently available en passant file.
    #[inline(always)]
    pub(crate) fn set_en_passant(&mut self, en_passant: Option<File>) {
        self.zobrist_hash ^= get_en_passant_zobrist(self.en_passant);
        self.zobrist_hash ^= get_en_passant_zobrist(en_passant);
        self.en_passant = en_passant;
    }
    // Sets the state of castling rights.
    #[inline(always)]
    pub(crate) fn set_castling_rights(&mut self, castling_rights: CastlingRights) {
        self.zobrist_hash ^= get_castling_zobrist(self.castling_rights);
        self.zobrist_hash ^= get_castling_zobrist(castling_rights);
        self.castling_rights = castling_rights;
    }
    // Adds a piece to the board.
    //
    // # Panics
    // If trying to add the piece to an already occupied square
    #[inline(always)]
    pub(crate) fn add_color_piece(&mut self, color: Color, piece: Piece, sq: Square) {
        debug_assert!(!self.board.get_occupance().has_square(sq));
        self.zobrist_hash ^= get_square_zobrist(color, piece, sq);
        self.board.mask_or(color, piece, BitBoard::from(sq));
    }
    // Removes a piece from the board.
    //
    // # Panics
    // - if trying to remove an unoccupied square
    // - if the `sq` contains the piece of different type or color than specified.
    #[inline(always)]
    pub(crate) fn remove_color_piece(&mut self, color: Color, piece: Piece, sq: Square) {
        debug_assert!(
            self.board.get_piece(piece).has_square(sq)
                && self.board.get_color(color).has_square(sq)
        );
        self.zobrist_hash ^= get_square_zobrist(color, piece, sq);
        self.board.mask_and(color, piece, !BitBoard::from(sq));
    }
    // Moves a piece on the board.
    //
    // # Panics
    // - if trying to move a piece from an unoccupied square
    // - if `from` contains a piece with a different type or color than specified
    // - if `to` is occupied
    #[inline(always)]
    pub(crate) fn move_color_piece(
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
    // Sets the color of the player that is about to make a turn.
    #[inline(always)]
    pub(crate) fn set_turn(&mut self, turn: Color) {
        self.zobrist_hash ^= get_turn_zobrist(self.turn);
        self.zobrist_hash ^= get_turn_zobrist(turn);
        self.turn = turn;
    }
    // Sets the current move index.
    pub(crate) fn set_move_index(&mut self, move_index: u32) {
        self.move_index = move_index;
    }
    // Sets the move index when the 50 move rule was last reset.
    pub(crate) fn set_move_index_rule_50(&mut self, move_index: u32) {
        self.move_index_rule_50 = move_index;
    }
    // Pushes a ply on top of the history vector.
    pub(crate) fn push_history(&mut self, ply: PlyHistory) {
        self.history.push(ply);
    }
    // Pops the last ply from the history vector.
    //
    // # Panics
    // Panics if the history vector is empty.
    pub(crate) fn pop_history(&mut self) -> PlyHistory {
        self.history.pop().unwrap()
    }
}

impl fmt::Display for Game {
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
            self.turn,
            self.castling_rights,
            if let Some(en_passant) = self.en_passant {
                &format!("on {en_passant} file")
            } else {
                "N/A"
            },
            self.move_index - self.move_index_rule_50,
            self.zobrist_hash,
            self.board,
        )
    }
}
