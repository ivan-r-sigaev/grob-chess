use perft::{count_leaves, perft_leaves, PerftValues};
use position::prelude::Position;

mod perft {
    use position::prelude::{ChessMoveHint, Position};

    use crate::game::move_list::MoveList;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct PerftValues {
        pub nodes: u128,
        pub captures: u128,
        pub ep: u128,
        pub castles: u128,
        pub promotions: u128,
        pub checks: u128,
        pub checkmates: u128,
    }

    impl PerftValues {
        pub fn new() -> Self {
            Self {
                nodes: 0,
                captures: 0,
                ep: 0,
                castles: 0,
                promotions: 0,
                checks: 0,
                checkmates: 0,
            }
        }
        pub fn add_move(&mut self, hint: ChessMoveHint) {
            self.nodes += 1;
            if hint.is_capture() {
                self.captures += 1;
            }
            if hint == ChessMoveHint::EnPassantCapture {
                self.ep += 1;
            }
            if hint == ChessMoveHint::KingCastle || hint == ChessMoveHint::QueenCastle {
                self.castles += 1;
            }
            if hint.is_promotion() {
                self.promotions += 1;
            }
        }
        pub fn add_check(&mut self) {
            self.checks += 1;
        }
        pub fn add_checkmate(&mut self) {
            self.checkmates += 1;
        }
    }

    pub fn perft_leaves(mut position: Position, depth: u8) -> PerftValues {
        let mut data = PerftValues::new();
        collect_perft(&mut position, &mut MoveList::empty(), &mut data, depth - 1);
        data
    }
    pub fn count_leaves(mut position: Position, depth: u8) -> u128 {
        count_legal_moves(&mut position, &mut MoveList::empty(), depth - 1)
    }

    fn can_move(position: &mut Position, move_list: &mut MoveList) -> bool {
        move_list.generate_moves(position);

        while let Some(next_move) = move_list.pop_move() {
            let unmove = position.make_move(next_move);

            if !position.was_check_ignored() {
                position.unmake_move(unmove);
                move_list.pop_group();
                return true;
            }

            position.unmake_move(unmove);
        }

        move_list.pop_group();
        false
    }

    fn count_legal_moves(position: &mut Position, move_list: &mut MoveList, depth: u8) -> u128 {
        let mut count = 0u128;
        move_list.generate_moves(position);

        while let Some(next_move) = move_list.pop_move() {
            let unmove = position.make_move(next_move);

            if !position.was_check_ignored() {
                if depth == 0 {
                    count += 1;
                } else {
                    count += count_legal_moves(position, move_list, depth - 1);
                }
            }

            position.unmake_move(unmove);
        }

        move_list.pop_group();
        count
    }

    fn collect_perft(
        position: &mut Position,
        move_list: &mut MoveList,
        data: &mut PerftValues,
        depth: u8,
    ) {
        move_list.generate_moves(position);

        while let Some(next_move) = move_list.pop_move() {
            let unmove = position.make_move(next_move);

            if !position.was_check_ignored() {
                if depth == 0 {
                    data.add_move(next_move.hint);
                    if position.is_check() {
                        data.add_check();
                        if !can_move(position, move_list) {
                            data.add_checkmate();
                        }
                    }
                } else {
                    collect_perft(position, move_list, data, depth - 1);
                }
            }

            position.unmake_move(unmove);
        }

        move_list.pop_group();
    }
}

const INITIAL_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
const KIWIPETE_FEN: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
const CPW_DEBUG_3_FEN: &str = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
const CPW_MIRROR_W_FEN: &str = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
const CPW_MIRROR_B_FEN: &str = "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1";

const CPW_DEBUG_5_FEN: &str = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";

fn perft(fen: &str, depth: u8, expected: PerftValues) {
    let position = Position::try_from_fen(fen).unwrap();
    let values = perft_leaves(position, depth);
    println!("explicit => {values:?}");
    assert_eq!(values, expected);
}

fn test_move_count(fen: &str, depth: u8, expected: u128) {
    let position = Position::try_from_fen(fen).unwrap();
    let values = count_leaves(position, depth);
    assert_eq!(values, expected);
}

#[test]
fn cpw_debug_5_move_count_depth_1() {
    test_move_count(CPW_DEBUG_5_FEN, 1, 44);
}

#[test]
fn cpw_debug_5_move_count_depth_2() {
    test_move_count(CPW_DEBUG_5_FEN, 2, 1_486);
}

#[test]
fn cpw_debug_5_move_count_depth_3() {
    test_move_count(CPW_DEBUG_5_FEN, 3, 62_379);
}

#[test]
fn cpw_debug_5_move_count_depth_4() {
    test_move_count(CPW_DEBUG_5_FEN, 4, 2_103_487);
}

#[test]
#[ignore = "reason: slow test"]
fn cpw_debug_5_move_count_depth_5() {
    test_move_count(CPW_DEBUG_5_FEN, 5, 89_941_194);
}

#[test]
fn cpw_mirror_w_perft_depth_1() {
    perft(
        CPW_MIRROR_W_FEN,
        1,
        PerftValues {
            nodes: 6,
            captures: 0,
            ep: 0,
            castles: 0,
            promotions: 0,
            checks: 0,
            checkmates: 0,
        },
    )
}

#[test]
fn cpw_mirror_b_perft_depth_1() {
    perft(
        CPW_MIRROR_B_FEN,
        1,
        PerftValues {
            nodes: 6,
            captures: 0,
            ep: 0,
            castles: 0,
            promotions: 0,
            checks: 0,
            checkmates: 0,
        },
    )
}

#[test]
fn cpw_mirror_w_perft_depth_2() {
    perft(
        CPW_MIRROR_W_FEN,
        2,
        PerftValues {
            nodes: 264,
            captures: 87,
            ep: 0,
            castles: 6,
            promotions: 48,
            checks: 10,
            checkmates: 0,
        },
    )
}

#[test]
fn cpw_mirror_b_perft_depth_2() {
    perft(
        CPW_MIRROR_B_FEN,
        2,
        PerftValues {
            nodes: 264,
            captures: 87,
            ep: 0,
            castles: 6,
            promotions: 48,
            checks: 10,
            checkmates: 0,
        },
    )
}

#[test]
fn cpw_mirror_w_perft_depth_3() {
    perft(
        CPW_MIRROR_W_FEN,
        3,
        PerftValues {
            nodes: 9467,
            captures: 1021,
            ep: 4,
            castles: 0,
            promotions: 120,
            checks: 38,
            checkmates: 22,
        },
    )
}

#[test]
fn cpw_mirror_b_perft_depth_3() {
    perft(
        CPW_MIRROR_B_FEN,
        3,
        PerftValues {
            nodes: 9467,
            captures: 1021,
            ep: 4,
            castles: 0,
            promotions: 120,
            checks: 38,
            checkmates: 22,
        },
    )
}

#[test]
fn cpw_mirror_w_perft_depth_4() {
    perft(
        CPW_MIRROR_W_FEN,
        4,
        PerftValues {
            nodes: 422333,
            captures: 131393,
            ep: 0,
            castles: 7795,
            promotions: 60032,
            checks: 15492,
            checkmates: 5,
        },
    )
}

#[test]
fn cpw_mirror_b_perft_depth_4() {
    perft(
        CPW_MIRROR_B_FEN,
        4,
        PerftValues {
            nodes: 422333,
            captures: 131393,
            ep: 0,
            castles: 7795,
            promotions: 60032,
            checks: 15492,
            checkmates: 5,
        },
    )
}

#[test]
fn cpw_debug_3_perft_depth_1() {
    perft(
        CPW_DEBUG_3_FEN,
        1,
        PerftValues {
            nodes: 14,
            captures: 1,
            ep: 0,
            castles: 0,
            promotions: 0,
            checks: 2,
            checkmates: 0,
        },
    )
}

#[test]
fn cpw_debug_3_perft_depth_2() {
    perft(
        CPW_DEBUG_3_FEN,
        2,
        PerftValues {
            nodes: 191,
            captures: 14,
            ep: 0,
            castles: 0,
            promotions: 0,
            checks: 10,
            checkmates: 0,
        },
    )
}

#[test]
fn cpw_debug_3_perft_depth_3() {
    perft(
        CPW_DEBUG_3_FEN,
        3,
        PerftValues {
            nodes: 2812,
            captures: 209,
            ep: 2,
            castles: 0,
            promotions: 0,
            checks: 267,
            checkmates: 0,
        },
    )
}

#[test]
fn cpw_debug_3_perft_depth_4() {
    perft(
        CPW_DEBUG_3_FEN,
        4,
        PerftValues {
            nodes: 43238,
            captures: 3348,
            ep: 123,
            castles: 0,
            promotions: 0,
            checks: 1680,
            checkmates: 17,
        },
    )
}

#[test]
fn cpw_debug_3_perft_depth_5() {
    perft(
        CPW_DEBUG_3_FEN,
        5,
        PerftValues {
            nodes: 674624,
            captures: 52051,
            ep: 1165,
            castles: 0,
            promotions: 0,
            checks: 52950,
            checkmates: 0,
        },
    )
}

#[test]
#[ignore = "reason: slow test"]
fn cpw_debug_3_perft_depth_6() {
    perft(
        CPW_DEBUG_3_FEN,
        6,
        PerftValues {
            nodes: 11030083,
            captures: 940350,
            ep: 33325,
            castles: 0,
            promotions: 7552,
            checks: 452473,
            checkmates: 2733,
        },
    )
}

#[test]
fn kiwipete_perft_depth_1() {
    perft(
        KIWIPETE_FEN,
        1,
        PerftValues {
            nodes: 48,
            captures: 8,
            ep: 0,
            castles: 2,
            promotions: 0,
            checks: 0,
            checkmates: 0,
        },
    )
}

#[test]
fn kiwipete_perft_depth_2() {
    perft(
        KIWIPETE_FEN,
        2,
        PerftValues {
            nodes: 2039,
            captures: 351,
            ep: 1,
            castles: 91,
            promotions: 0,
            checks: 3,
            checkmates: 0,
        },
    )
}

#[test]
fn kiwipete_perft_depth_3() {
    perft(
        KIWIPETE_FEN,
        3,
        PerftValues {
            nodes: 97862,
            captures: 17102,
            ep: 45,
            castles: 3162,
            promotions: 0,
            checks: 993,
            checkmates: 1,
        },
    )
}

#[test]
fn kiwipete_perft_depth_4() {
    perft(
        KIWIPETE_FEN,
        4,
        PerftValues {
            nodes: 4085603,
            captures: 757163,
            ep: 1929,
            castles: 128013,
            promotions: 15172,
            checks: 25523,
            checkmates: 43,
        },
    )
}

#[test]
fn initial_position_perft_depth_1() {
    perft(
        INITIAL_FEN,
        1,
        PerftValues {
            nodes: 20,
            captures: 0,
            ep: 0,
            castles: 0,
            promotions: 0,
            checks: 0,
            checkmates: 0,
        },
    )
}

#[test]
fn initial_position_perft_depth_2() {
    perft(
        INITIAL_FEN,
        2,
        PerftValues {
            nodes: 400,
            captures: 0,
            ep: 0,
            castles: 0,
            promotions: 0,
            checks: 0,
            checkmates: 0,
        },
    )
}

#[test]
fn initial_position_perft_depth_3() {
    perft(
        INITIAL_FEN,
        3,
        PerftValues {
            nodes: 8_902,
            captures: 34,
            ep: 0,
            castles: 0,
            promotions: 0,
            checks: 12,
            checkmates: 0,
        },
    )
}

#[test]
fn initial_position_perft_depth_4() {
    perft(
        INITIAL_FEN,
        4,
        PerftValues {
            nodes: 197_281,
            captures: 1_576,
            ep: 0,
            castles: 0,
            promotions: 0,
            checks: 469,
            checkmates: 8,
        },
    )
}

#[test]
fn initial_position_perft_depth_5() {
    perft(
        INITIAL_FEN,
        5,
        PerftValues {
            nodes: 4_865_609,
            captures: 82_719,
            ep: 258,
            castles: 0,
            promotions: 0,
            checks: 27_351,
            checkmates: 347,
        },
    )
}

#[test]
#[ignore = "reason: slow test"]
fn initial_position_perft_depth_6() {
    perft(
        INITIAL_FEN,
        6,
        PerftValues {
            nodes: 119_060_324,
            captures: 2_812_008,
            ep: 5248,
            castles: 0,
            promotions: 0,
            checks: 809_099,
            checkmates: 10_828,
        },
    )
}
