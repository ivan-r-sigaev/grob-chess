use std::fmt;

use crate::bitboard::BitBoard;
use crate::indexing::{Color, File, Piece, PosDiag, Rank, Square};
use strum::{EnumCount, IntoEnumIterator, VariantArray};

/// Current state of all the pieces on the chess board.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Board {
    /*
    White  = 0,
    Black  = 1,
    Pawn   = 2,
    Bishop = 3,
    Knight = 4,
    Rook   = 5,
    Queen  = 6,
    King   = 7
    */
    boards: [BitBoard; 8],
}

impl Board {
    // TODO: it may be better to add a constructor from FEN.
    /// Constructs an empty board.
    ///
    /// # Returns
    /// `Self` - an empty board.
    #[inline(always)]
    #[must_use]
    pub fn empty() -> Self {
        Self {
            boards: [BitBoard::EMPTY; 8],
        }
    }

    /// Returns the bitboard with all the pieces of the given color.
    ///
    /// # Arguments
    /// * `color` - the given color
    ///
    /// # Returns
    /// * `BitBoard` - the bitboard with all the pieces of the given color
    #[inline(always)]
    #[must_use]
    pub fn get_color(&self, color: Color) -> BitBoard {
        self.boards[color as usize]
    }

    /// Returns the bitboard with all the pieces currently present on the board.
    ///
    /// # Returns
    /// * `BitBoard` - the bitboard with all the pieces currently present on the board
    #[inline(always)]
    #[must_use]
    pub fn get_occupance(&self) -> BitBoard {
        self.get_color(Color::White) | self.get_color(Color::Black)
    }

    /// Returns the bitboard with all the unoccupied squares.
    ///
    /// # Returns
    /// * `BitBoard` - the bitboard with all the unoccupied squares
    #[inline(always)]
    #[must_use]
    pub fn get_empty(&self) -> BitBoard {
        !self.get_occupance()
    }

    /// Returns the bitboard with all the pieces of the given piece type.
    ///
    /// # Arguments
    /// * `piece` - the given piece type
    ///
    /// # Returns
    /// * `BitBoard` - the bitboard with all the pieces of the given piece type
    #[inline(always)]
    #[must_use]
    pub fn get_piece(&self, piece: Piece) -> BitBoard {
        self.boards[piece as usize + 2]
    }

    /// Returns the bitboard with all the pieces that are BOTH the of given piece type and of the given color.
    ///
    /// # Arguments
    /// * `color` - the given color
    /// * `piece` - the given piece type
    ///
    /// # Returns
    /// * `BitBoard` - the bitboard with all the pieces that are BOTH the of given piece type and of the given color
    #[inline(always)]
    #[must_use]
    pub fn get_color_piece(&self, color: Color, piece: Piece) -> BitBoard {
        self.get_color(color) & self.get_piece(piece)
    }

    /// Returns the piece type placed on the given square (or `None` if the square is empty).
    ///
    /// # Arguments
    /// * `sq` - the given square
    ///
    /// # Returns
    /// `Option<Piece>`:
    /// - `Some(piece: Piece)` - the piece type placed on the given square
    /// - `None` - if the square is empty
    #[inline(always)]
    #[must_use]
    pub fn get_piece_at(&self, sq: Square) -> Option<Piece> {
        let bb = BitBoard::from(sq);
        if !(bb & self.get_piece(Piece::Pawn)).is_empty() {
            Some(Piece::Pawn)
        } else if !(bb & self.get_piece(Piece::Bishop)).is_empty() {
            Some(Piece::Bishop)
        } else if !(bb & self.get_piece(Piece::Knight)).is_empty() {
            Some(Piece::Knight)
        } else if !(bb & self.get_piece(Piece::Rook)).is_empty() {
            Some(Piece::Rook)
        } else if !(bb & self.get_piece(Piece::Queen)).is_empty() {
            Some(Piece::Queen)
        } else if !(bb & self.get_piece(Piece::King)).is_empty() {
            Some(Piece::King)
        } else {
            None
        }
    }

    /// Returns the color of the piece placed on the given square (or `None` if the square is empty).
    ///
    /// # Arguments
    /// * `sq` - the given square
    ///
    /// # Returns
    /// `Option<Color>`:
    /// - `Some(piece: Color)` - the color of the piece placed on the given square
    /// - `None` - if the square is empty
    #[inline(always)]
    #[must_use]
    pub fn get_color_at(&self, sq: Square) -> Option<Color> {
        let bb = BitBoard::from(sq);
        if !(bb & self.get_color(Color::White)).is_empty() {
            Some(Color::White)
        } else if !(bb & self.get_color(Color::Black)).is_empty() {
            Some(Color::Black)
        } else {
            None
        }
    }

    // /// Returns the bitboard with all the pieces that attack (put pressure on) the given square.
    // ///
    // /// # Arguments
    // /// * `sq` - the given square
    // ///
    // /// # Returns
    // /// `BitBoard` - the bitboard with all the pieces that attack (put pressure on) the given square
    // #[inline(always)]
    // #[must_use]
    // pub fn get_attackers_to(&self, sq: Square) -> BitBoard {
    //     let occ = self.get_occupance();

    //     BitBoard::pawn_attacks(sq, Color::White) & self.get_color_piece(Color::Black, Piece::Pawn)
    //         | BitBoard::pawn_attacks(sq, Color::Black)
    //             & self.get_color_piece(Color::White, Piece::Pawn)
    //         | BitBoard::knight_attacks(sq) & self.get_piece(Piece::Knight)
    //         | BitBoard::king_attacks(sq) & self.get_piece(Piece::King)
    //         | BitBoard::bishop_attacks(occ, sq) & self.get_bishop_sliders()
    //         | BitBoard::rook_attacks(occ, sq) & self.get_rook_sliders()
    // }

    /// Returns the bitboard with all the pieces of a given color that attack (put pressure on) the given square.
    ///
    /// # Arguments
    /// * `sq` - the given square
    /// * `color` - the given color
    ///
    /// # Returns
    /// `BitBoard` - the bitboard with all the pieces of a given color that attack (put pressure on) the given square
    #[inline(always)]
    #[must_use]
    pub fn get_color_attackers_to(&self, sq: Square, color: Color) -> BitBoard {
        let occ = self.get_occupance();

        self.get_color(color)
            & (BitBoard::pawn_attacks(sq, !color) & self.get_piece(Piece::Pawn)
                | BitBoard::knight_attacks(sq) & self.get_piece(Piece::Knight)
                | BitBoard::king_attacks(sq) & self.get_piece(Piece::King)
                | BitBoard::bishop_attacks(occ, sq) & self.get_bishop_sliders()
                | BitBoard::rook_attacks(occ, sq) & self.get_rook_sliders())
    }
}

impl Board {
    /// Returns the bitboard with all the queens and bishops.
    ///
    /// # Returns
    /// `BitBoard` - the bitboard with all the queens and bishops
    #[inline(always)]
    #[must_use]
    pub fn get_bishop_sliders(&self) -> BitBoard {
        self.get_piece(Piece::Queen) | self.get_piece(Piece::Bishop)
    }

    /// Returns the bitboard with all the queens and bishops of the given color.
    ///
    /// # Arguments
    /// * `color` - the given color
    ///
    /// # Returns
    /// `BitBoard` - the bitboard with all the queens and bishops of the given color
    #[inline(always)]
    #[must_use]
    pub fn get_color_bishop_sliders(&self, color: Color) -> BitBoard {
        self.get_color(color) & self.get_bishop_sliders()
    }

    /// Returns the bitboard with all the queens and rooks.
    ///
    /// # Returns
    /// `BitBoard` - the bitboard with all the queens and rooks
    #[inline(always)]
    #[must_use]
    pub fn get_rook_sliders(&self) -> BitBoard {
        self.get_piece(Piece::Queen) | self.get_piece(Piece::Rook)
    }

    /// Returns the bitboard with all the rooks and bishops of the given color.
    ///
    /// # Arguments
    /// * `color` - the given color
    ///
    /// # Returns
    /// `BitBoard` - the bitboard with all the queens and rooks of the given color
    #[inline(always)]
    #[must_use]
    pub fn get_color_rook_sliders(&self, color: Color) -> BitBoard {
        self.get_color(color) & self.get_rook_sliders()
    }

    // TODO: This function relies on a failable assumtion that the king exists.
    /// Returns whether the king of the given color is currently in check.
    ///
    /// # Arguments
    /// * `color` - the given color
    ///
    /// # Returns
    /// `bool` - whether the king of the given color is currently in check
    #[inline(always)]
    #[must_use]
    pub fn is_king_in_check(&self, color: Color) -> bool {
        !self
            .get_color_attackers_to(
                BitBoard::bit_scan_forward(self.get_color_piece(color, Piece::King))
                    .expect("king does not exist"),
                !color,
            )
            .is_empty()
    }
}

impl Board {
    /// Places (or replaces) pieces of the given color on the squares specified by the mask with the given piece type.
    ///
    /// # Preconditions
    /// 
    /// The user of the function is responsible for not trying to overwrite the squares that contain
    /// the opposite color, which will result in doubly colored pieces.
    ///
    /// # Arguments
    /// * `color` - the color of pieces
    /// * `piece` - the type of the pieces to place
    /// * `mask` - the mask where to place (or replace) the pieces
    #[inline(always)]
    pub fn mask_or(&mut self, color: Color, piece: Piece, mask: BitBoard) {
        self.boards[piece as usize + 2] |= mask;
        self.boards[color as usize] |= mask;
    }

    /// Removes all pieces of the given color and type on the squares NOT specified by the mask.
    ///
    /// # Preconditions
    /// 
    /// The user of the function is responsible for not trying to remove the pieces of a different color
    /// than specified by the mask, which will result in colored squares without a piece type.
    ///
    /// # Arguments
    /// * `color` - the color of the pieces
    /// * `piece` - the type of the pieces
    /// * `mask` - the mask specifying what pieces to keep
    #[inline(always)]
    pub fn mask_and(&mut self, color: Color, piece: Piece, mask: BitBoard) {
        self.boards[piece as usize + 2] &= mask;
        self.boards[color as usize] &= mask;
    }

    /// Toggles all the pieces of the given color and type on the squares specified by the mask.
    ///
    /// # Preconditions
    /// 
    /// The user of the function is responsible for not trying to toggle the pieces of a different color
    /// or a different piece type than specified by the mask, which will result in one of the following:
    /// - multicolored pieces
    /// - multityped pieces
    /// - uncolored pieces
    /// - colored squares without a piece type
    /// - severe headaches and vomiting
    /// - immediate heat death of the universe
    /// - \[REDACTED\]
    ///
    /// # Arguments
    /// * `color` - the color of the pieces
    /// * `piece` - the type of the pieces
    /// * `mask` - the mask specifying what pieces to toggle
    #[inline(always)]
    pub fn mask_xor(&mut self, color: Color, piece: Piece, mask: BitBoard) {
        self.boards[piece as usize + 2] ^= mask;
        self.boards[color as usize] ^= mask;
    }
}

impl std::fmt::Debug for Board {
    /// Formats board for debug purposes.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{{\
                white: {{\n{}}},\n\
                black: {{\n{}}},\n\
                pawn: {{\n{}}},\n\
                bishop: {{\n{}}},\n\
                knight: {{\n{}}},\n\
                rook: {{\n{}}},\n\
                queen: {{\n{}}},\n\
                king: {{\n{}}},\n\
            }}",
            self.get_color(Color::White),
            self.get_color(Color::Black),
            self.get_piece(Piece::Pawn),
            self.get_piece(Piece::Bishop),
            self.get_piece(Piece::Knight),
            self.get_piece(Piece::Rook),
            self.get_piece(Piece::Queen),
            self.get_piece(Piece::King),
        )
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut drawing = String::new();
        for rank in Rank::iter().rev() {
            drawing += "  ";
            for file in File::iter() {
                let sq = Square::new(rank, file);
                let piece = self.get_piece_at(sq);
                let color = self.get_color_at(sq);
                if let Some((color, piece)) = color.zip(piece) {
                    drawing += &format!("{color}{piece}");
                } else {
                    drawing += "__";
                }
                drawing += " ";
            }
            drawing += "\n"
        }
        write!(f, "Chess board {{\n{drawing}}}")
    }
}

impl BitBoard {
    /// Returns the quiet move of the pawn of a given color from a given square.
    ///
    /// # Arguments
    /// * `from` - the square from which the move is generated
    /// * `color` - color of the pawn
    ///
    /// # Returns
    /// `BitBoard` - the resulting move
    #[inline(always)]
    #[must_use]
    pub const fn pawn_quiet(from: Square, color: Color) -> BitBoard {
        let bb = BitBoard::from_square(from);
        match color {
            Color::White => bb.up(),
            Color::Black => bb.down(),
        }
    }

    /// Returns the attack move of the pawn of a given color from a given square.
    ///
    /// # Arguments
    /// * `from` - the square from which the move is generated
    /// * `color` - color of the pawn
    ///
    /// # Returns
    /// `BitBoard` - the resulting move
    #[inline(always)]
    #[must_use]
    pub const fn pawn_attacks(from: Square, color: Color) -> BitBoard {
        let bb = Self::pawn_quiet(from, color);
        bb.left().bitor(bb.right())
    }

    /// Returns the move of a kinght from a given square.
    ///
    /// # Arguments
    /// * `from` - the square from which the move is generated
    ///
    /// # Returns
    /// `BitBoard` - the resulting move
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

    /// Returns the move of a king from a given square.
    ///
    /// # Arguments
    /// * `from` - the square from which the move is generated
    ///
    /// # Returns
    /// `BitBoard` - the resulting move
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

    /// Returns the move of a bishop from a given square.
    ///
    /// # Arguments
    /// * `from` - the square from which the move is generated
    /// * `occupance` - the occupance to account for
    ///
    /// # Returns
    /// `BitBoard` - the resulting move
    #[inline(always)]
    #[must_use]
    pub const fn bishop_attacks(occupance: BitBoard, from: Square) -> BitBoard {
        Self::pos_diag_attacks(from, occupance).bitor(Self::neg_diag_attacks(from, occupance))
    }

    /// Returns the move of a rook from a given square.
    ///
    /// # Arguments
    /// * `from` - the square from which the move is generated
    /// * `occupance` - the occupance to account for
    ///
    /// # Returns
    /// `BitBoard` - the resulting move
    #[inline(always)]
    #[must_use]
    pub const fn rook_attacks(occupance: BitBoard, from: Square) -> BitBoard {
        Self::rank_attacks(from, occupance).bitor(Self::file_attack(from, occupance))
    }

    /// Returns the move of a queen from a given square.
    ///
    /// # Arguments
    /// * `from` - the square from which the move is generated
    /// * `occupance` - the occupance to account for
    ///
    /// # Returns
    /// `BitBoard` - the resulting move
    #[inline(always)]
    #[must_use]
    pub const fn queen_attacks(occupance: BitBoard, from: Square) -> BitBoard {
        Self::bishop_attacks(occupance, from).bitor(Self::rook_attacks(occupance, from))
    }

    /// Returns the combined quiet moves of pawns from a given square.
    ///
    /// # Arguments
    /// * `pawns` - the squares from which the move is generated
    /// * `empty` - the unoccupied squares
    /// * `color` - the color of the pawns
    ///
    /// # Returns
    /// `BitBoard` - the resulting move
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
