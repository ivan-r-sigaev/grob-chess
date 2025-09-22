mod collect {
    use grob_core::{ChessMoveHint, Game, GameTreeWalker, MoveOrdering};

    /// Number of leaf nodes visited by the perft.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
    pub struct PerftResult {
        pub nodes: u64,
        pub captures: u64,
        pub ep: u64,
        pub castles: u64,
        pub promotions: u64,
        pub checks: u64,
        pub checkmates: u64,
    }

    pub fn perft_detailed(fen: &str, depth: u8, expected: PerftResult) {
        let mut result = PerftResult::default();
        let mut game = Game::try_from_fen(fen).expect("Incorrect FEN!");
        collect_detailed(&mut result, &mut game.walk(), depth - 1);
        assert_eq!(result, expected, "Incorrect perft results!");
    }

    pub fn perft(fen: &str, depth: u8, expected: u64) {
        let mut result = 0;
        let mut game = Game::try_from_fen(fen).expect("Incorrect FEN!");
        collect(&mut result, &mut game.walk(), depth - 1);
        assert_eq!(result, expected, "Incorrect perft results!");
    }

    fn collect_detailed(result: &mut PerftResult, node: &mut GameTreeWalker, depth: u8) {
        _ = node.for_each_legal_child_node(MoveOrdering::default(), |node, chess_move| {
            if depth != 0 {
                collect_detailed(result, node, depth - 1);
                return;
            }
            result.add_move(chess_move.hint());

            if !node.game().is_check() {
                return;
            }
            result.add_check();

            if !node.check_ending().is_right() {
                return;
            }
            result.add_checkmate();
        })
    }

    fn collect(result: &mut u64, node: &mut GameTreeWalker, depth: u8) {
        _ = node.for_each_legal_child_node(MoveOrdering::default(), |node, _| {
            if depth != 0 {
                collect(result, node, depth - 1);
                return;
            }
            *result += 1;
        })
    }

    impl PerftResult {
        fn add_move(&mut self, hint: ChessMoveHint) {
            self.nodes += 1;
            if matches!(hint, ChessMoveHint::EnPassantCapture) {
                self.ep += 1;
            }
            if matches!(hint, ChessMoveHint::KingCastle | ChessMoveHint::QueenCastle) {
                self.castles += 1;
            }
            if hint.is_capture() {
                self.captures += 1;
            }
            if hint.is_promotion() {
                self.promotions += 1;
            }
        }
        fn add_check(&mut self) {
            self.checks += 1;
        }
        fn add_checkmate(&mut self) {
            self.checkmates += 1;
        }
    }
}

pub use collect::{perft, perft_detailed, PerftResult};

mod initial {
    use crate::{perft_detailed, PerftResult};

    const FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    #[test]
    fn depth1() {
        let expected = PerftResult {
            nodes: 20,
            captures: 0,
            ep: 0,
            castles: 0,
            promotions: 0,
            checks: 0,
            checkmates: 0,
        };
        perft_detailed(FEN, 1, expected);
    }

    #[test]
    fn depth2() {
        let expected = PerftResult {
            nodes: 400,
            captures: 0,
            ep: 0,
            castles: 0,
            promotions: 0,
            checks: 0,
            checkmates: 0,
        };
        perft_detailed(FEN, 2, expected);
    }

    #[test]
    fn depth3() {
        let expected = PerftResult {
            nodes: 8_902,
            captures: 34,
            ep: 0,
            castles: 0,
            promotions: 0,
            checks: 12,
            checkmates: 0,
        };
        perft_detailed(FEN, 3, expected);
    }

    #[test]
    fn depth4() {
        let expected = PerftResult {
            nodes: 197_281,
            captures: 1_576,
            ep: 0,
            castles: 0,
            promotions: 0,
            checks: 469,
            checkmates: 8,
        };
        perft_detailed(FEN, 4, expected);
    }

    #[test]
    fn depth5() {
        let expected = PerftResult {
            nodes: 4_865_609,
            captures: 82_719,
            ep: 258,
            castles: 0,
            promotions: 0,
            checks: 27_351,
            checkmates: 347,
        };
        perft_detailed(FEN, 5, expected);
    }

    #[test]
    #[ignore = "reason: slow test"]
    fn depth6() {
        let expected = PerftResult {
            nodes: 119_060_324,
            captures: 2_812_008,
            ep: 5248,
            castles: 0,
            promotions: 0,
            checks: 809_099,
            checkmates: 10_828,
        };
        perft_detailed(FEN, 6, expected);
    }
}

mod kiwipete {
    use crate::{perft_detailed, PerftResult};

    const FEN: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";

    #[test]
    fn depth1() {
        let expected = PerftResult {
            nodes: 48,
            captures: 8,
            ep: 0,
            castles: 2,
            promotions: 0,
            checks: 0,
            checkmates: 0,
        };
        perft_detailed(FEN, 1, expected);
    }

    #[test]
    fn depth2() {
        let expected = PerftResult {
            nodes: 2039,
            captures: 351,
            ep: 1,
            castles: 91,
            promotions: 0,
            checks: 3,
            checkmates: 0,
        };
        perft_detailed(FEN, 2, expected);
    }

    #[test]
    fn depth3() {
        let expected = PerftResult {
            nodes: 97862,
            captures: 17102,
            ep: 45,
            castles: 3162,
            promotions: 0,
            checks: 993,
            checkmates: 1,
        };
        perft_detailed(FEN, 3, expected);
    }

    #[test]
    fn depth4() {
        let expected = PerftResult {
            nodes: 4085603,
            captures: 757163,
            ep: 1929,
            castles: 128013,
            promotions: 15172,
            checks: 25523,
            checkmates: 43,
        };
        perft_detailed(FEN, 4, expected);
    }
}

mod mirrored {
    use crate::{perft_detailed, PerftResult};

    const FEN_WHITE: &str = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
    const FEN_BLACK: &str = "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1";

    #[test]
    fn depth1() {
        let expected = PerftResult {
            nodes: 6,
            captures: 0,
            ep: 0,
            castles: 0,
            promotions: 0,
            checks: 0,
            checkmates: 0,
        };
        perft_detailed(FEN_WHITE, 1, expected);
        perft_detailed(FEN_BLACK, 1, expected);
    }

    #[test]
    fn depth2() {
        let expected = PerftResult {
            nodes: 264,
            captures: 87,
            ep: 0,
            castles: 6,
            promotions: 48,
            checks: 10,
            checkmates: 0,
        };
        perft_detailed(FEN_WHITE, 2, expected);
        perft_detailed(FEN_BLACK, 2, expected);
    }

    #[test]
    fn depth3() {
        let expected = PerftResult {
            nodes: 9467,
            captures: 1021,
            ep: 4,
            castles: 0,
            promotions: 120,
            checks: 38,
            checkmates: 22,
        };
        perft_detailed(FEN_WHITE, 3, expected);
        perft_detailed(FEN_BLACK, 3, expected);
    }

    #[test]
    fn depth4() {
        let expected = PerftResult {
            nodes: 422333,
            captures: 131393,
            ep: 0,
            castles: 7795,
            promotions: 60032,
            checks: 15492,
            checkmates: 5,
        };
        perft_detailed(FEN_WHITE, 4, expected);
        perft_detailed(FEN_BLACK, 4, expected);
    }
}

mod cpw3 {
    use crate::{perft_detailed, PerftResult};

    const FEN: &str = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";

    #[test]
    fn depth1() {
        let expected = PerftResult {
            nodes: 14,
            captures: 1,
            ep: 0,
            castles: 0,
            promotions: 0,
            checks: 2,
            checkmates: 0,
        };
        perft_detailed(FEN, 1, expected);
    }

    #[test]
    fn depth2() {
        let expected = PerftResult {
            nodes: 191,
            captures: 14,
            ep: 0,
            castles: 0,
            promotions: 0,
            checks: 10,
            checkmates: 0,
        };
        perft_detailed(FEN, 2, expected);
    }

    #[test]
    fn depth3() {
        let expected = PerftResult {
            nodes: 2812,
            captures: 209,
            ep: 2,
            castles: 0,
            promotions: 0,
            checks: 267,
            checkmates: 0,
        };
        perft_detailed(FEN, 3, expected);
    }

    #[test]
    fn depth4() {
        let expected = PerftResult {
            nodes: 43238,
            captures: 3348,
            ep: 123,
            castles: 0,
            promotions: 0,
            checks: 1680,
            checkmates: 17,
        };
        perft_detailed(FEN, 4, expected);
    }

    #[test]
    fn depth5() {
        let expected = PerftResult {
            nodes: 674624,
            captures: 52051,
            ep: 1165,
            castles: 0,
            promotions: 0,
            checks: 52950,
            checkmates: 0,
        };
        perft_detailed(FEN, 5, expected);
    }

    #[test]
    #[ignore = "reason: slow test"]
    fn depth6() {
        let expected = PerftResult {
            nodes: 11030083,
            captures: 940350,
            ep: 33325,
            castles: 0,
            promotions: 7552,
            checks: 452473,
            checkmates: 2733,
        };
        perft_detailed(FEN, 6, expected);
    }
}

mod cpw5 {
    use crate::perft;

    const FEN: &str = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";

    #[test]
    fn depth1() {
        perft(FEN, 1, 44);
    }

    #[test]
    fn depth2() {
        perft(FEN, 2, 1_486);
    }

    #[test]
    fn depth3() {
        perft(FEN, 3, 62_379);
    }

    #[test]
    fn depth4() {
        perft(FEN, 4, 2_103_487);
    }

    #[test]
    #[ignore = "reason: slow test"]
    fn depth5() {
        perft(FEN, 5, 89_941_194);
    }
}
