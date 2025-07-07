use bitflags::bitflags;  // This dependency is a pure utility. It may be removed if needed.
use crate::game::position::board::Color;

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct CastlingRights: u8 {
        const WHITE_KING = 1 << 0;
        const WHITE_QUEEN = 1 << 1;
        const BLACK_KING = 1 << 2;
        const BLACK_QUEEN = 1 << 3;
    }
}

impl CastlingRights {
    #[inline(always)]
    pub fn kingside(color: Color) -> CastlingRights {
        return if color == Color::White {
            CastlingRights::WHITE_KING
        } else {
            CastlingRights::BLACK_KING
        };
    }
    #[inline(always)]
    pub fn queenside(color: Color) -> CastlingRights {
        return if color == Color::White {
            CastlingRights::WHITE_QUEEN
        } else {
            CastlingRights::BLACK_QUEEN
        };
    }
    #[inline(always)]
    pub fn both_sides(color: Color) -> CastlingRights {
        return if color == Color::White {
            CastlingRights::WHITE_QUEEN | CastlingRights::WHITE_KING
        } else {
            CastlingRights::BLACK_QUEEN | CastlingRights::BLACK_KING
        };
    }
}
