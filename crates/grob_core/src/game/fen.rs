use std::collections::VecDeque;

use crate::{
    bitboard::BitBoard,
    board::Board,
    castling_rights::CastlingRights,
    game::Game,
    pieces::{piece_from_fen, Color},
    position::Position,
    square::{File, Square},
};

impl Game {
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
}
