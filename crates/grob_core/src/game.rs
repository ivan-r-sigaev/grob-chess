pub use lan::LanMove;
pub use movegen::{ChessMove, ChessMoveHint, PackedChessMove};
pub use walker::{GameEnding, GameTreeWalker, MoveOrdering};

use std::{collections::VecDeque, num::NonZeroU64};

use crate::{
    bitboard::BitBoard,
    board::Board,
    castling_rights::CastlingRights,
    game::make::ChessUnmove,
    pieces::{piece_from_fen, Color},
    position::Position,
    square::{File, Square},
};

pub mod walker;

mod lan;
mod make;
mod movegen;

/// A chess position.
#[derive(Debug, Clone)]
pub struct Game {
    history: Vec<PlyHistory>,
    position: Position,
    move_index_rule_50: u32,
    move_index: u32,
}

#[derive(Debug, Clone, Copy)]
struct PlyHistory {
    hash: NonZeroU64,
    unmove: ChessUnmove,
}

impl Game {
    /// Returns the initial position for the standard chess game.
    pub fn initial_position() -> Self {
        const INITIAL_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        Self::try_from_fen(INITIAL_FEN).unwrap()
    }
    /// Tries to parse a positioin from FEN.
    pub fn try_from_fen(fen: &str) -> Option<Self> {
        let mut words: VecDeque<&str> = fen.split_whitespace().collect();

        let board = {
            let fen = words.pop_front()?;
            let rows = fen.split('/').collect::<Vec<_>>();
            if rows.len() != 8 {
                return None;
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
                        let (color, piece) = piece_from_fen(ch)?;
                        board.mask_or(color, piece, BitBoard::from(sq));
                    }
                    sq = sq.shifted(1);
                    row_len += 1;
                    if row_len > 8 {
                        return None;
                    }
                }
                if row_len < 8 {
                    return None;
                }
            }

            board
        };

        let turn = words.pop_front().and_then(|s| {
            Some(match s {
                "w" => Color::White,
                "b" => Color::Black,
                _ => return None,
            })
        })?;

        let castling = words
            .pop_front()
            .and_then(CastlingRights::from_fen_segment)?;

        let en_passant = match words.pop_front()? {
            "-" => None,
            s => Some(s.parse::<File>().ok()?),
        };

        let position = Position::from_parts(board, turn, castling, en_passant)?;

        let hm = words.pop_front().and_then(|s| s.parse::<u32>().ok())?;

        let fm = words.pop_front().and_then(|s| s.parse::<u32>().ok())?;

        if fm == 0 {
            return None;
        }

        let move_index = (fm - 1) * 2 + (turn == Color::Black) as u32;

        if hm > move_index {
            return None;
        }

        let move_index_rule_50 = move_index - hm;

        if !words.is_empty() {
            return None;
        }

        let history = Vec::new();

        Some(Game {
            position,
            move_index,
            move_index_rule_50,
            history,
        })
    }
    /// Returns a hash for the current position.
    #[must_use]
    pub fn zobrist(&self) -> NonZeroU64 {
        self.position.zobrist()
    }
    /// Returns the possible en passant target file if available or `None`.
    #[must_use]
    pub fn en_passant(&self) -> Option<File> {
        self.position.en_passant()
    }
    /// Returns the state of castling rights.
    #[must_use]
    pub fn castling_rights(&self) -> CastlingRights {
        self.position.castling_rights()
    }
    /// Returns a reference to the position's board.
    #[must_use]
    pub fn board(&self) -> &Board {
        self.position.board()
    }
    /// Returns the color of the player who is about to make a turn.
    #[must_use]
    pub fn turn(&self) -> Color {
        self.position.turn()
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
    fn can_capture_king(&self) -> bool {
        self.board().is_king_in_check(!self.turn())
    }
    /// Increments the move index and returns the previous state of halfmove clock.
    fn next_move_index(&mut self, reset_hm_clock: bool) -> u32 {
        let res = self.move_index - self.move_index_rule_50;
        self.move_index += 1;
        if reset_hm_clock {
            self.move_index_rule_50 = self.move_index;
        }
        res
    }
    /// Decrements the move index and set the state of halfmove clock.
    fn prev_move_index(&mut self, hm_clock_state: u32) {
        self.move_index = self.move_index.strict_sub(1);
        self.move_index_rule_50 = self.move_index.strict_sub(hm_clock_state);
    }
}
