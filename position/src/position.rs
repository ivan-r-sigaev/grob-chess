//! Position
//!
//! This module provedes types related to position representation and move generation.

pub use crate::castling_rights::CastlingRights;
pub use crate::move_generation::{ChessMove, ChessMoveHint, ChessUnmove, LanMove, PackedChessMove};
pub use crate::position_hash::PositionHash;

use crate::bitboard::BitBoard;
use crate::board::Board;
use crate::pieces::{Color, Piece};
use crate::square::{File, Square};

use std::error::Error;
use std::fmt::{self, Display};
use std::hash::Hash;

use zobrist::{get_castling_zobrist, get_en_passant_zobrist, get_square_zobrist, get_turn_zobrist};

mod zobrist;

/// A chess position.
#[derive(Debug, Clone, Copy)]
pub struct Position {
    board: Board,
    turn: Color,
    castling_rights: CastlingRights,
    en_passant: Option<File>,
    move_index_rule_50: u32,
    move_index: u32,
    zobrist_hash: u64,
    //piece_scores: [PieceScore; 2],
}

impl PartialEq for Position {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.zobrist_hash == other.zobrist_hash  // Compare hash to maybe get a faster comparison on average.
        && self.board == other.board
        && self.turn == other.turn
        && self.castling_rights == other.castling_rights
        && self.en_passant == other.en_passant
        // Should this compare move_index and halfmove_index?
    }
}

impl Eq for Position {}

impl Hash for Position {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.board.hash(state);
        self.turn.hash(state);
        self.castling_rights.hash(state);
        self.en_passant.hash(state);
        self.move_index_rule_50.hash(state);
        self.move_index.hash(state);
        self.zobrist_hash.hash(state);
    }
}

/// An error that originated from [FEN] parsing.
///
/// [fen]: https://www.chessprogramming.org/Forsyth-Edwards_Notation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParseFenError {
    BadFenSize,
    BadRowCount,
    BadRowSize,
    BadCastlingRights,
    BadEnPassant,
    BadHalfmoveClock,
    BadFullmoveClock,
    UnknownCharacter,
}

impl Display for ParseFenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Error for ParseFenError {}

impl Position {
    /// Returns the initial position for a normal chess game.
    pub fn initial_position() -> Self {
        const INITIAL_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        Self::try_from_fen(INITIAL_FEN).unwrap()
    }

    /// Tries to construct a new position from [FEN].
    ///
    /// # Examples
    /// ```rust
    /// use position::position::{Position, ParseFenError};
    ///
    /// const INITIAL_POSITION_FEN: &str =
    ///     "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    /// let initial_position_from_fen = Position::try_from_fen(INITIAL_POSITION_FEN);
    /// assert_eq!(initial_position_from_fen, Ok(Position::initial_position()))
    /// ```
    ///
    /// [fen]: https://www.chessprogramming.org/Forsyth-Edwards_Notation
    pub fn try_from_fen(fen: &str) -> Result<Self, ParseFenError> {
        let fen_parts: Vec<&str> = fen.split_whitespace().collect();
        if fen_parts.len() != 6 {
            return Err(ParseFenError::BadFenSize);
        }

        let position_parts: Vec<&str> = fen_parts[0].split('/').collect();
        if position_parts.len() != 8 {
            return Err(ParseFenError::BadRowCount);
        }

        let mut bitboard = Board::empty();
        let mut zobrist_hash = 0;
        let mut sq: Square = Square::A1;
        for y in (0..8).rev() {
            let mut x = 0;
            for ch in position_parts[y].chars() {
                if ('1'..='8').contains(&ch) {
                    let increment = u8::try_from(ch).unwrap() - u8::try_from('1').unwrap();
                    x += increment;
                    sq = sq.shifted(increment as i8);
                } else if ch.is_ascii_alphabetic() {
                    let lower = ch.to_ascii_lowercase();
                    let color = if ch.is_ascii_lowercase() {
                        Color::Black
                    } else {
                        Color::White
                    };

                    let piece = if lower == 'k' {
                        Piece::King
                    } else if lower == 'q' {
                        Piece::Queen
                    } else if lower == 'r' {
                        Piece::Rook
                    } else if lower == 'b' {
                        Piece::Bishop
                    } else if lower == 'n' {
                        Piece::Knight
                    } else if lower == 'p' {
                        Piece::Pawn
                    } else {
                        return Err(ParseFenError::UnknownCharacter);
                    };

                    bitboard.mask_or(color, piece, BitBoard::from(sq));
                    zobrist_hash ^= get_square_zobrist(color, piece, sq);
                } else {
                    return Err(ParseFenError::UnknownCharacter);
                }
                sq = sq.shifted(1);
                x += 1;
                if x > 8 {
                    return Err(ParseFenError::BadFenSize);
                }
            }
            if x < 8 {
                return Err(ParseFenError::BadFenSize);
            }
        }

        let turn = if fen_parts[1] == "w" {
            Color::White
        } else if fen_parts[1] == "b" {
            Color::Black
        } else {
            return Err(ParseFenError::UnknownCharacter);
        };

        zobrist_hash ^= get_turn_zobrist(turn);

        let mut castling_rights = CastlingRights::empty();
        if fen_parts[2] != "-" {
            let mut remaining_len = fen_parts[2].len();
            if fen_parts[2].contains('K') {
                remaining_len -= 1;
                castling_rights |= CastlingRights::WHITE_KING;
            }
            if fen_parts[2].contains('Q') {
                remaining_len -= 1;
                castling_rights |= CastlingRights::WHITE_QUEEN;
            }
            if fen_parts[2].contains('k') {
                remaining_len -= 1;
                castling_rights |= CastlingRights::BLACK_KING;
            }
            if fen_parts[2].contains('q') {
                remaining_len -= 1;
                castling_rights |= CastlingRights::BLACK_QUEEN;
            }
            if remaining_len != 0 {
                return Err(ParseFenError::BadCastlingRights);
            }
        }
        zobrist_hash ^= get_castling_zobrist(castling_rights);

        let en_passant;
        if fen_parts[3] == "-" {
            en_passant = None;
        } else {
            if fen_parts[3].chars().count() != 2 {
                return Err(ParseFenError::BadEnPassant);
            }
            let col = fen_parts[3].chars().nth(0).unwrap();
            let row = fen_parts[3].chars().nth(1).unwrap();
            if row != '6' || row != '3' {
                return Err(ParseFenError::BadEnPassant);
            }

            if col == 'a' {
                en_passant = Some(File::A);
            } else if col == 'b' {
                en_passant = Some(File::B);
            } else if col == 'c' {
                en_passant = Some(File::C);
            } else if col == 'd' {
                en_passant = Some(File::D);
            } else if col == 'e' {
                en_passant = Some(File::E);
            } else if col == 'f' {
                en_passant = Some(File::F);
            } else if col == 'g' {
                en_passant = Some(File::G);
            } else if col == 'h' {
                en_passant = Some(File::H);
            } else {
                return Err(ParseFenError::BadEnPassant);
            }
        }
        zobrist_hash ^= get_en_passant_zobrist(en_passant);

        let hm = fen_parts[4].parse::<u32>();
        if hm.is_err() {
            return Err(ParseFenError::BadHalfmoveClock);
        }
        let halfmove_clock = hm.unwrap();

        let fm = fen_parts[5].parse::<u32>();
        if fm.is_err() {
            return Err(ParseFenError::BadFullmoveClock);
        }
        let move_index = fm.unwrap();
        let halfmove_index = move_index - halfmove_clock;

        // let piece_scores: [PieceScore; 2] = [
        //     board.get_piece_score(Color::try_from(0)),
        //     board.get_piece_score(Color::Black)
        // ];

        Ok(Position {
            board: bitboard,
            turn,
            castling_rights,
            en_passant,
            move_index,
            move_index_rule_50: halfmove_index,
            zobrist_hash,
            //piece_scores,
        })
    }
}

impl Position {
    /// Returns a hash for the current position.
    #[inline(always)]
    #[must_use]
    pub fn position_hash(&self) -> PositionHash {
        PositionHash::new(self.zobrist_hash)
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
}

impl Position {
    /// Sets the currently available en passant file.
    #[inline(always)]
    pub fn set_en_passant(&mut self, en_passant: Option<File>) {
        self.zobrist_hash ^= get_en_passant_zobrist(self.en_passant);
        self.zobrist_hash ^= get_en_passant_zobrist(en_passant);
        self.en_passant = en_passant;
    }

    /// Sets the state of castling rights.
    #[inline(always)]
    pub fn set_castling_rights(&mut self, castling_rights: CastlingRights) {
        self.zobrist_hash ^= get_castling_zobrist(self.castling_rights);
        self.zobrist_hash ^= get_castling_zobrist(castling_rights);
        self.castling_rights = castling_rights;
    }

    /// Adds a piece to the board.
    ///
    /// # Panics
    /// If trying to add the piece to an already occupied square
    #[inline(always)]
    pub fn add_color_piece(&mut self, color: Color, piece: Piece, sq: Square) {
        debug_assert!(!self.board.get_occupance().has_square(sq));
        self.zobrist_hash ^= get_square_zobrist(color, piece, sq);
        self.board.mask_or(color, piece, BitBoard::from(sq));
    }

    /// Removes a piece from the board.
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
        self.zobrist_hash ^= get_square_zobrist(color, piece, sq);
        self.board.mask_and(color, piece, !BitBoard::from(sq));
    }

    /// Moves a piece on the board.
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
        debug_assert!(!self.board.get_occupance().has_square(to));
        self.zobrist_hash ^= get_square_zobrist(color, piece, from);
        self.zobrist_hash ^= get_square_zobrist(color, piece, to);
        self.board
            .mask_xor(color, piece, BitBoard::from(from) | BitBoard::from(to));
    }

    /// Sets the color of the player that is about to make a turn.
    #[inline(always)]
    pub fn set_turn(&mut self, turn: Color) {
        self.zobrist_hash ^= get_turn_zobrist(self.turn);
        self.zobrist_hash ^= get_turn_zobrist(turn);
        self.turn = turn;
    }

    /// Sets the current move index.
    pub fn set_move_index(&mut self, move_index: u32) {
        self.move_index = move_index;
    }

    /// Sets the move index when the 50 move rule was last reset.
    pub fn set_move_index_rule_50(&mut self, move_index: u32) {
        self.move_index_rule_50 = move_index;
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            concat!(
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
