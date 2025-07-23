use crate::bitboard::{BitBoard, File, Square};
use crate::board::{Board, Color, Piece};
pub use castling_rights::CastlingRights;

use std::error::Error;
use std::fmt::{self, Display};
use std::hash::Hash;
use std::mem::size_of;
use std::ops::Rem;

use zobrist::{get_castling_zobrist, get_en_passant_zobrist, get_square_zobrist, get_turn_zobrist};

mod castling_rights;
mod zobrist;

/// A chess position.
#[derive(Debug, Clone, Copy)]
pub struct Position {
    board: Board,
    turn: Color,
    castling_rights: CastlingRights,
    en_passant: Option<File>,
    halfmove_clock: u32,
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
    }
}

impl Eq for Position {}

impl Hash for Position {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.board.hash(state);
        self.turn.hash(state);
        self.castling_rights.hash(state);
        self.en_passant.hash(state);
        self.halfmove_clock.hash(state);
        self.zobrist_hash.hash(state);
    }
}

/// An error that had occured while parsing [FEN].
///
/// [fen]: https://www.chessprogramming.org/Forsyth-Edwards_Notation
#[derive(Debug, Clone)]
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
    /// Try to construct a new position from [FEN].
    ///
    /// # Arguments
    /// * `fen` - FEN text
    ///
    /// # Returns
    /// `Result<Self, ParseFenError>`:
    /// - `Ok(position: Self)` - the parsed position
    /// - `Error(error: ParseFenError)` - the error that occured while parsing
    ///
    /// # Examples
    /// ```rust
    /// use position::prelude::{Position, ParseFenError};
    ///
    /// let initial_position_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    /// _ = Position::try_from_fen(initial_position_fen)?;
    /// Ok::<(), ParseFenError>(())
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

        // let piece_scores: [PieceScore; 2] = [
        //     board.get_piece_score(Color::try_from(0)),
        //     board.get_piece_score(Color::Black)
        // ];

        Ok(Position {
            board: bitboard,
            turn,
            castling_rights,
            en_passant,
            halfmove_clock,
            zobrist_hash,
            //piece_scores,
        })
    }
}

/// Unique hash generated from a chess position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PositionHash(u64);

impl Rem<usize> for PositionHash {
    type Output = usize;

    fn rem(self, rhs: usize) -> Self::Output {
        const MAX: u64 = if size_of::<u64>() > size_of::<usize>() {
            usize::MAX as u64
        } else {
            u64::MAX
        };
        (self.0 & MAX) as usize % rhs
    }
}

impl fmt::Display for PositionHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Position {
    /// Returns a hash for the current position.
    ///
    /// # Returns
    /// `PositionHash` - a hash for the current position
    #[inline(always)]
    #[must_use]
    pub fn position_hash(&self) -> PositionHash {
        PositionHash(self.zobrist_hash)
    }

    /// Returns the currently available en passant file (or `None` if not available).
    ///
    /// # Returns
    /// `Option<File>`:
    /// - `Some(file: File)` - the file of the currently available en passant
    /// - `None` - if en passant is not currently available
    #[inline(always)]
    #[must_use]
    pub fn en_passant(&self) -> Option<File> {
        self.en_passant
    }

    /// Returns the current castling rights of a position.
    ///
    /// # Returns
    /// `CastlingRights` - the current castling rights of a position
    #[inline(always)]
    #[must_use]
    pub fn castling_rights(&self) -> CastlingRights {
        self.castling_rights
    }

    /// Returns a reference to the position's board.
    ///
    /// # Returns
    /// `Board` - a reference to the position's board
    #[inline(always)]
    #[must_use]
    pub fn board(&self) -> &Board {
        &self.board
    }

    /// Returns the color of the player who is about to make a turn.
    ///
    /// # Returns
    /// `Color` - the color of the player who is about to make a turn
    #[inline(always)]
    #[must_use]
    pub fn turn(&self) -> Color {
        self.turn
    }

    /// Returns the current state of the [halfmove clock].
    ///
    /// # Returns
    /// `u32` - the current amount of halfmoves
    ///
    /// [halfmove clock]: https://www.chessprogramming.org/Halfmove_Clock
    #[inline(always)]
    #[must_use]
    pub fn halfmove_clock(&self) -> u32 {
        self.halfmove_clock
    }
}

impl Position {
    /// Sets the currently available en passant file.
    ///
    /// # Arguments
    /// * `en_passant` - the value to set
    #[inline(always)]
    pub fn set_en_passant(&mut self, en_passant: Option<File>) {
        self.zobrist_hash ^= get_en_passant_zobrist(self.en_passant);
        self.zobrist_hash ^= get_en_passant_zobrist(en_passant);
        self.en_passant = en_passant;
    }

    /// Sets the currently available castling rights.
    ///
    /// # Arguments
    /// * `castling_rights` - the value to set
    #[inline(always)]
    pub fn set_castling_rights(&mut self, castling_rights: CastlingRights) {
        self.zobrist_hash ^= get_castling_zobrist(self.castling_rights);
        self.zobrist_hash ^= get_castling_zobrist(castling_rights);
        self.castling_rights = castling_rights;
    }

    /// Adds a piece to the board.
    ///
    /// # Arguments
    /// `color` - the color of the piece to add
    /// `piece` - the type of the piece to add
    /// `sq` - the square where to add the piece
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
    /// # Arguments
    /// `color` - the color of the piece to remove
    /// `piece` - the type of the piece to remove
    /// `sq` - the square where to remove the piece
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
    /// # Arguments
    /// `color` - the color of the piece to move
    /// `piece` - the type of the piece to move
    /// `from` - the square from where to move the piece
    /// `to` - the square where to move the piece
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
    ///
    /// # Arguments
    /// `turn` - the color of the player that is about to make a turn
    #[inline(always)]
    pub fn set_turn(&mut self, turn: Color) {
        self.zobrist_hash ^= get_turn_zobrist(self.turn);
        self.zobrist_hash ^= get_turn_zobrist(turn);
        self.turn = turn;
    }

    /// Sets the current state of [halfmove clock].
    ///
    /// # Arguments
    /// `halfmove_clock` - the amount of halfmoves
    ///
    /// [halfmove clock]: https://www.chessprogramming.org/Halfmove_Clock
    #[inline(always)]
    pub fn set_halfmove_clock(&mut self, halfmove_clock: u32) {
        self.halfmove_clock = halfmove_clock;
    }
}

impl Position {
    /// Returns whether kingside castling is NOT allowed for a given color.
    ///
    /// # Returns
    /// `bool` - whether kingside castling is NOT allowed for a given color
    #[inline(always)]
    #[must_use]
    pub fn is_kingside_castling_prohibited(&self, color: Color) -> bool {
        // TODO: remove crights when rook is taken instead of checking for it's existence
        let w_empty = BitBoard::from(Square::F1) | BitBoard::from(Square::G1);
        let b_empty = BitBoard::from(Square::F8) | BitBoard::from(Square::G8);
        !self
            .castling_rights
            .contains(CastlingRights::kingside(self.turn))
            || (self.board.get_color_piece(color, Piece::Rook)
                & BitBoard::from(if color == Color::White {
                    Square::H1
                } else {
                    Square::H8
                }))
            .is_empty()
            || !(self.board.get_occupance()
                & if color == Color::White {
                    w_empty
                } else {
                    b_empty
                })
            .is_empty()
            || !(self.board.get_color_attackers_to(
                if color == Color::White {
                    Square::F1
                } else {
                    Square::F8
                },
                !color,
            ))
            .is_empty()
            || !(self.board.get_color_attackers_to(
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
    pub fn is_queenside_castling_prohibited(&self, color: Color) -> bool {
        // TODO: remove crights when rook is taken instead of checking for it's existence
        let w_empty =
            BitBoard::from(Square::B1) | BitBoard::from(Square::C1) | BitBoard::from(Square::D1);
        let b_empty =
            BitBoard::from(Square::B8) | BitBoard::from(Square::C8) | BitBoard::from(Square::D8);
        !self
            .castling_rights
            .contains(CastlingRights::queenside(self.turn))
            || (self.board.get_color_piece(color, Piece::Rook)
                & BitBoard::from(if color == Color::White {
                    Square::A1
                } else {
                    Square::A8
                }))
            .is_empty()
            || !(self.board.get_occupance()
                & if color == Color::White {
                    w_empty
                } else {
                    b_empty
                })
            .is_empty()
            || !(self.board.get_color_attackers_to(
                if color == Color::White {
                    Square::C1
                } else {
                    Square::C8
                },
                !color,
            ))
            .is_empty()
            || !(self.board.get_color_attackers_to(
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

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            concat!(
                "Chess position {{\n",
                "  turn: {}\n",
                "  castling rights: {}\n",
                "  available en passant: {}\n",
                "  moves since last capture/pawn move/check: {}\n",
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
            self.halfmove_clock,
            self.zobrist_hash,
            self.board,
        )
    }
}
