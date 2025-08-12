use crate::board::{BitBoard, Color, File, PosDiag, Rank, Square};
use strum::{EnumCount, VariantArray};

impl BitBoard {
    /// Returns the quiet move of the pawn of a given color from a given square.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn pawn_quiet(from: Square, color: Color) -> BitBoard {
        let bb = BitBoard::from_square(from);
        match color {
            Color::White => bb.up(),
            Color::Black => bb.down(),
        }
    }

    /// Returns the attack move of the pawn of a given color from a given square.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn pawn_attacks(from: Square, color: Color) -> BitBoard {
        let bb = Self::pawn_quiet(from, color);
        bb.left().bitor(bb.right())
    }

    /// Returns the move of a kinght from a given square.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn knight_attacks(from: Square) -> BitBoard {
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

    /// Returns the move of a king from a given square.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn king_attacks(from: Square) -> BitBoard {
        let bb = BitBoard::from_square(from);
        let tmp = bb.left().bitor(bb.right());
        tmp.bitor(tmp.up())
            .bitor(tmp.down())
            .bitor(bb.up())
            .bitor(bb.down())
    }

    /// Returns the move of a bishop from a given square.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn bishop_attacks(occupance: BitBoard, from: Square) -> BitBoard {
        Self::pos_diag_attacks(from, occupance).bitor(Self::neg_diag_attacks(from, occupance))
    }

    /// Returns the move of a rook from a given square.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn rook_attacks(occupance: BitBoard, from: Square) -> BitBoard {
        Self::rank_attacks(from, occupance).bitor(Self::file_attack(from, occupance))
    }

    /// Returns the move of a queen from a given square.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn queen_attacks(occupance: BitBoard, from: Square) -> BitBoard {
        Self::bishop_attacks(occupance, from).bitor(Self::rook_attacks(occupance, from))
    }

    /// Returns the combined quiet moves of pawns from a given square.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn pawn_pushes(pawns: BitBoard, empty: BitBoard, color: Color) -> BitBoard {
        match color {
            Color::White => pawns.up(),
            Color::Black => pawns.down(),
        }
        .bitand(empty)
    }
    #[inline(always)]
    #[must_use]
    const fn up(self) -> Self {
        self.shl(File::COUNT as u8)
    }
    #[inline(always)]
    #[must_use]
    const fn down(self) -> Self {
        self.shr(File::COUNT as u8)
    }
    #[inline(always)]
    #[must_use]
    const fn right(self) -> Self {
        self.bitand(Self::from_file(File::H).not()).shl(1)
    }
    #[inline(always)]
    #[must_use]
    const fn left(self) -> Self {
        self.bitand(Self::from_file(File::A).not()).shr(1)
    }
    #[inline(always)]
    #[must_use]
    const fn fill_up(self) -> Self {
        self.mul(Self::from_file(File::A))
    }
    #[inline(always)]
    #[must_use]
    const fn attack_right(mut self, occupance: BitBoard) -> Self {
        let empty = occupance.not();
        self.bitor_assign(self.right().bitand(empty)); // 1
        self.bitor_assign(self.right().bitand(empty)); // 2
        self.bitor_assign(self.right().bitand(empty)); // 3
        self.bitor_assign(self.right().bitand(empty)); // 4
        self.bitor_assign(self.right().bitand(empty)); // 5
        self.bitor_assign(self.right().bitand(empty)); // 6
        self.bitand(empty).bitor(self.right()) // 7
    }
    #[inline(always)]
    #[must_use]
    const fn attack_left(mut self, occupance: BitBoard) -> Self {
        let empty = occupance.not();
        self.bitor_assign(self.left().bitand(empty)); // 1
        self.bitor_assign(self.left().bitand(empty)); // 2
        self.bitor_assign(self.left().bitand(empty)); // 3
        self.bitor_assign(self.left().bitand(empty)); // 4
        self.bitor_assign(self.left().bitand(empty)); // 5
        self.bitor_assign(self.left().bitand(empty)); // 6
        self.bitand(empty).bitor(self.left()) // 7
    }
    #[inline(always)]
    #[must_use]
    const fn attack_up(mut self, occupance: BitBoard) -> Self {
        let empty = occupance.not();
        self.bitor_assign(self.up().bitand(empty)); // 1
        self.bitor_assign(self.up().bitand(empty)); // 2
        self.bitor_assign(self.up().bitand(empty)); // 3
        self.bitor_assign(self.up().bitand(empty)); // 4
        self.bitor_assign(self.up().bitand(empty)); // 5
        self.bitor_assign(self.up().bitand(empty)); // 6
        self.bitand(empty).bitor(self.up()) // 7
    }
    #[inline(always)]
    #[must_use]
    const fn attack_down(mut self, occupance: BitBoard) -> Self {
        let empty = occupance.not();
        self.bitor_assign(self.down().bitand(empty)); // 1
        self.bitor_assign(self.down().bitand(empty)); // 2
        self.bitor_assign(self.down().bitand(empty)); // 3
        self.bitor_assign(self.down().bitand(empty)); // 4
        self.bitor_assign(self.down().bitand(empty)); // 5
        self.bitor_assign(self.down().bitand(empty)); // 6
        self.bitand(empty).bitor(self.down()) // 7
    }
    #[inline(always)]
    #[must_use]
    const fn rank_to_reversed_file(self) -> Self {
        self.mul(Self::from_pos_diag(PosDiag::A1H8))
            .shr(7)
            .bitand(Self::from_file(File::A))
    }
    #[inline(always)]
    #[must_use]
    const fn file_to_reversed_rank(self) -> Self {
        self.mul(Self::from_pos_diag(PosDiag::A1H8))
            .shr(File::COUNT as u8 * (Rank::COUNT as u8 - 1))
    }
    #[inline(always)]
    #[must_use]
    const fn project_on_rank(self) -> Self {
        self.fill_up()
            .shr(File::COUNT as u8 * (Rank::COUNT as u8 - 1))
    }
    const KINDERGARTEN_OCCUPANCY_MAX: u8 = 64;
    #[inline(always)]
    #[must_use]
    const fn into_kindergarten_occupancy(self) -> u8 {
        assert!(self.bitand(BitBoard::from_rank(Rank::R1).not()).is_empty());
        self.bitand(BitBoard::from_square(Square::H1).not())
            .shr(1)
            .0 as u8
    }
    #[inline(always)]
    #[must_use]
    const fn from_kindergarten_occupancy_as_rank(file: File, kg_occupancy: u8) -> BitBoard {
        const LOOKUP: [[BitBoard; BitBoard::KINDERGARTEN_OCCUPANCY_MAX as usize]; File::COUNT] = {
            let mut result =
                [[BitBoard::EMPTY; BitBoard::KINDERGARTEN_OCCUPANCY_MAX as usize]; File::COUNT];
            let mut i = 0;
            while i < File::COUNT {
                let file = File::VARIANTS[i];
                let mut kg_occupancy = 0;
                while kg_occupancy < BitBoard::KINDERGARTEN_OCCUPANCY_MAX {
                    let kg_occupancy_bb = BitBoard(kg_occupancy as u64).shl(1);
                    let slider = BitBoard::from_square(Square::new(Rank::R1, file));
                    result[file as usize][kg_occupancy as usize] = slider
                        .attack_left(kg_occupancy_bb)
                        .bitor(slider.attack_right(kg_occupancy_bb))
                        .fill_up();
                    kg_occupancy += 1;
                }
                i += 1;
            }
            result
        };
        LOOKUP[file as usize][kg_occupancy as usize]
    }
    #[inline(always)]
    #[must_use]
    const fn from_kindergarten_occupancy_as_file(rank: Rank, kg_occupancy_rev: u8) -> BitBoard {
        const LOOKUP: [[BitBoard; BitBoard::KINDERGARTEN_OCCUPANCY_MAX as usize]; Rank::COUNT] = {
            let mut result =
                [[BitBoard::EMPTY; BitBoard::KINDERGARTEN_OCCUPANCY_MAX as usize]; Rank::COUNT];
            let mut i = 0;
            while i < Rank::COUNT {
                let rank = Rank::VARIANTS[i];
                let mut kg_occupancy_rev = 0;
                while kg_occupancy_rev < BitBoard::KINDERGARTEN_OCCUPANCY_MAX {
                    let kg_occupancy_rev_bb = BitBoard(kg_occupancy_rev as u64).shl(1);
                    let occupancy_on_a_file = kg_occupancy_rev_bb.rank_to_reversed_file();
                    let slider = BitBoard::from_square(Square::new(rank, File::A));
                    result[rank as usize][kg_occupancy_rev as usize] = slider
                        .attack_up(occupancy_on_a_file)
                        .bitor(slider.attack_down(occupancy_on_a_file));
                    kg_occupancy_rev += 1;
                }
                i += 1;
            }
            result
        };
        LOOKUP[rank as usize][kg_occupancy_rev as usize]
    }

    #[inline(always)]
    #[must_use]
    const fn pos_diag_attacks(from: Square, occupance: BitBoard) -> BitBoard {
        let mask = BitBoard::from_pos_diag(from.pos_diag()).bitxor(BitBoard::from_square(from));
        let occ_6bit =
            BitBoard::into_kindergarten_occupancy(mask.bitand(occupance).project_on_rank());
        mask.bitand(BitBoard::from_kindergarten_occupancy_as_rank(
            from.file(),
            occ_6bit,
        ))
    }
    #[inline(always)]
    #[must_use]
    const fn neg_diag_attacks(from: Square, occupance: BitBoard) -> BitBoard {
        let mask = BitBoard::from_neg_diag(from.neg_diag()).bitxor(BitBoard::from_square(from));
        let occ_6bit =
            BitBoard::into_kindergarten_occupancy(mask.bitand(occupance).project_on_rank());
        mask.bitand(BitBoard::from_kindergarten_occupancy_as_rank(
            from.file(),
            occ_6bit,
        ))
    }
    #[inline(always)]
    #[must_use]
    const fn rank_attacks(from: Square, occupance: BitBoard) -> BitBoard {
        let mask = BitBoard::from_rank(from.rank()).bitxor(BitBoard::from_square(from));
        let occ_6bit =
            BitBoard::into_kindergarten_occupancy(mask.bitand(occupance).project_on_rank());
        mask.bitand(BitBoard::from_kindergarten_occupancy_as_rank(
            from.file(),
            occ_6bit,
        ))
    }
    #[inline(always)]
    #[must_use]
    const fn file_attack(from: Square, occupance: BitBoard) -> BitBoard {
        let rank = from.rank();
        let file = from.file();
        let file_occ = BitBoard::from_file(File::A).bitand(occupance.shr(file as u8));
        let rev_occ = file_occ.file_to_reversed_rank();
        let rev_occ_6bit = BitBoard::into_kindergarten_occupancy(rev_occ);
        BitBoard::from_kindergarten_occupancy_as_file(rank, rev_occ_6bit)
            .shl(file as u8)
            .bitand(BitBoard::from_square(from).not())
    }
}
