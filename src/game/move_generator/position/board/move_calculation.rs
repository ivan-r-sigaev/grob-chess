use crate::game::move_generator::position::board::{
    bitboard::{BitBoard, File, Square},
    Color,
};

impl BitBoard {
    #[inline(always)]
    #[must_use]
    pub const fn pawn_quiet(from: Square, color: Color) -> BitBoard {
        let bb = BitBoard::from_square(from);
        match color {
            Color::White => bb.up(),
            Color::Black => bb.down(),
        }
    }
    #[inline(always)]
    #[must_use]
    pub const fn pawn_attacks(from: Square, color: Color) -> BitBoard {
        let bb = Self::pawn_quiet(from, color);
        bb.left().bitor(bb.right())
    }
    #[inline(always)]
    #[must_use]
    pub const fn knight_attacks(from: Square) -> BitBoard {
        const MAGIC_L: BitBoard = BitBoard::EMPTY.not().left();
        const MAGIC_LL: BitBoard = MAGIC_L.left();
        const MAGIC_R: BitBoard = BitBoard::EMPTY.not().right();
        const MAGIC_RR: BitBoard = MAGIC_R.right();
        let bb = BitBoard::from_square(from);
        let l1 = bb.shr(1).bitand(MAGIC_L);
        let l2 = bb.shr(2).bitand(MAGIC_LL);
        let r1 = bb.shl(1).bitand(MAGIC_R);
        let r2 = bb.shl(2).bitand(MAGIC_RR);
        let h1 = l1.bitor(r1);
        let h2 = l2.bitor(r2);
        h1.shl(16)
            .bitor(h1.shr(16))
            .bitor(h2.shl(8))
            .bitor(h2.shr(8))
    }
    #[inline(always)]
    #[must_use]
    pub const fn king_attacks(from: Square) -> BitBoard {
        let bb = BitBoard::from_square(from);
        let tmp = bb.left().bitor(bb.right());
        tmp.bitor(tmp.up())
            .bitor(tmp.down())
            .bitor(bb.up())
            .bitor(bb.down())
    }
    #[inline(always)]
    #[must_use]
    pub const fn bishop_attacks(occupance: BitBoard, from: Square) -> BitBoard {
        Self::pos_diag_attacks(from, occupance).bitor(Self::neg_diag_attacks(from, occupance))
    }
    #[inline(always)]
    #[must_use]
    pub const fn rook_attacks(occupance: BitBoard, from: Square) -> BitBoard {
        Self::rank_attacks(from, occupance).bitor(Self::file_attack(from, occupance))
    }
    #[inline(always)]
    #[must_use]
    pub const fn queen_attacks(occupance: BitBoard, from: Square) -> BitBoard {
        Self::bishop_attacks(occupance, from).bitor(Self::rook_attacks(occupance, from))
    }
    #[inline(always)]
    #[must_use]
    pub const fn pawn_pushes(pawns: BitBoard, empty: BitBoard, color: Color) -> BitBoard {
        match color {
            Color::White => pawns.up(),
            Color::Black => pawns.down(),
        }
        .bitand(empty)
    }
    #[inline(always)]
    #[must_use]
    const fn pos_diag_attacks(from: Square, occupance: BitBoard) -> BitBoard {
        let mask =
            BitBoard::from_pos_diag(from.into_pos_diag()).bitxor(BitBoard::from_square(from));
        let occ_6bit =
            BitBoard::into_kindergarten_occupancy(mask.bitand(occupance).project_on_rank());
        mask.bitand(BitBoard::from_kindergarten_occupancy_as_rank(
            from.into_file(),
            occ_6bit,
        ))
    }
    #[inline(always)]
    #[must_use]
    const fn neg_diag_attacks(from: Square, occupance: BitBoard) -> BitBoard {
        let mask =
            BitBoard::from_neg_diag(from.into_neg_diag()).bitxor(BitBoard::from_square(from));
        let occ_6bit =
            BitBoard::into_kindergarten_occupancy(mask.bitand(occupance).project_on_rank());
        mask.bitand(BitBoard::from_kindergarten_occupancy_as_rank(
            from.into_file(),
            occ_6bit,
        ))
    }
    #[inline(always)]
    #[must_use]
    const fn rank_attacks(from: Square, occupance: BitBoard) -> BitBoard {
        let mask = BitBoard::from_rank(from.into_rank()).bitxor(BitBoard::from_square(from));
        let occ_6bit =
            BitBoard::into_kindergarten_occupancy(mask.bitand(occupance).project_on_rank());
        mask.bitand(BitBoard::from_kindergarten_occupancy_as_rank(
            from.into_file(),
            occ_6bit,
        ))
    }
    #[inline(always)]
    #[must_use]
    const fn file_attack(from: Square, occupance: BitBoard) -> BitBoard {
        let rank = from.into_rank();
        let file = from.into_file();
        let file_occ = BitBoard::from_file(File::A).bitand(occupance.shr(file as u8));
        let rev_occ = file_occ.file_to_reversed_rank();
        let rev_occ_6bit = BitBoard::into_kindergarten_occupancy(rev_occ);
        BitBoard::from_kindergarten_occupancy_as_file(rank, rev_occ_6bit)
            .shl(file as u8)
            .bitand(BitBoard::from_square(from).not())
    }
}
