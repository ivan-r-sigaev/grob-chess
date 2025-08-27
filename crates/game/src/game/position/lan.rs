use std::{fmt, str::FromStr};

use crate::{game::position::Position, ChessMove, Piece, Promotion, Square};

/// A chess move in a [LAN (Long Algebraic Notation)].
///
/// [LAN (Long Algebraic Notation)]:
/// https://www.chessprogramming.org/Algebraic_Chess_Notation#Long_Algebraic_Notation_.28LAN.29
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LanMove {
    /// The origin square.
    pub from: Square,
    /// The destination square.
    pub to: Square,
    /// The kind of promotion (if move is a promotion) or `None`.
    pub promotion: Option<Promotion>,
}

impl FromStr for LanMove {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (from_str, rest) = s.split_at_checked(2).ok_or(())?;
        let (to_str, maybe_promotion_str) = rest.split_at_checked(2).ok_or(())?;
        let from = from_str.parse::<Square>().map_err(|_| ())?;
        let to = to_str.parse::<Square>().map_err(|_| ())?;
        let promotion = match maybe_promotion_str {
            "" => None,
            promotion_str => Some(promotion_str.parse::<Promotion>().map_err(|_| ())?),
        };
        Ok(LanMove {
            from,
            to,
            promotion,
        })
    }
}

impl fmt::Display for LanMove {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.from, self.to)?;
        if let Some(promotion) = self.promotion {
            write!(f, "{promotion}")?;
        }
        Ok(())
    }
}

impl Position {
    /// Returns an equivalent `ChessMove` for a `LanMove` in this position.
    pub fn lan_move(&self, lan_move: LanMove) -> Option<ChessMove> {
        let piece = self.board().get_piece_at(lan_move.from)?;
        let mut result = None;
        let mut test_move = |chess_move: ChessMove| {
            if chess_move.lan() == lan_move {
                result = Some(chess_move);
            }
        };
        if self.board().get_piece_at(lan_move.to).is_some() {
            match piece {
                Piece::Pawn => self.push_pawn_attacks(&mut test_move),
                Piece::Bishop => self.push_bishop_attacks(&mut test_move),
                Piece::Knight => self.push_knight_attacks(&mut test_move),
                Piece::Rook => self.push_rook_attacks(&mut test_move),
                Piece::Queen => {
                    self.push_bishop_attacks(&mut test_move);
                    self.push_rook_attacks(&mut test_move);
                }
                Piece::King => self.push_king_attacks(&mut test_move),
            }
        } else {
            match piece {
                Piece::Pawn => {
                    // "to" is unoccupied for en passant.
                    self.push_pawn_attacks(&mut test_move);
                    self.push_pawn_quiets(&mut test_move);
                }
                Piece::Bishop => self.push_bishop_quiets(&mut test_move),
                Piece::Knight => self.push_knight_quiets(&mut test_move),
                Piece::Rook => self.push_rook_quiets(&mut test_move),
                Piece::Queen => {
                    self.push_bishop_quiets(&mut test_move);
                    self.push_rook_quiets(&mut test_move);
                }
                Piece::King => {
                    self.push_castlings(&mut test_move);
                    self.push_king_quiets(&mut test_move);
                }
            }
        }

        result
    }
}
