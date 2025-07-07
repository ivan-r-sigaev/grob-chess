mod data {
    use super::PerftValues;
    use crate::game::move_generator::*;
    use enum_iterator::all;

    #[derive(Default)]
    pub struct PerftData {
        move_count: [u128; 16],
        check_count: u128,
        checkmate_count: u128,
    }

    impl PerftData {
        pub fn new() -> PerftData {
            return PerftData {
                move_count: [0; 16],
                check_count: 0,
                checkmate_count: 0,
            };
        }
        pub fn get_move_count(&self, hint: MoveHint) -> u128 {
            return self.move_count[hint as usize];
        }
        pub fn add_move(&mut self, hint: MoveHint) {
            self.move_count[hint as usize] += 1;
        }
        pub fn add_check(&mut self) {
            self.check_count += 1;
        }
        pub fn add_checkmate(&mut self) {
            self.checkmate_count += 1;
        }
        pub fn count_moves(&self, p: fn(MoveHint) -> bool) -> u128 {
            let mut total = 0;
            for hint in all::<MoveHint>() {
                if p(hint) {
                    total += self.get_move_count(hint);
                }
            }
            return total;
        }
        pub fn get_check_count(&self) -> u128 {
            return self.check_count;
        }
        pub fn get_checkmate_count(&self) -> u128 {
            return self.checkmate_count;
        }
        pub fn to_perft_values(&self) -> PerftValues {
            let nodes = self.count_moves(|_| true);
            let captures = self.count_moves(|h| h.is_capture());
            let ep = self.count_moves(|h| h == MoveHint::EnPassantCapture);
            let castles =
                self.count_moves(|h| h == MoveHint::KingCastle || h == MoveHint::QueenCastle);
            let promotions = self.count_moves(|h| h.is_promotion());
            let checks = self.get_check_count();
            let checkmates = self.get_checkmate_count();

            return PerftValues {
                nodes,
                captures,
                ep,
                castles,
                promotions,
                checks,
                checkmates,
            };
        }
    }

    impl std::fmt::Debug for PerftData {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            use std::fmt::Write;
            let mut s = String::new();
            for hint in all::<MoveHint>() {
                let count = self.get_move_count(hint);
                if count != 0 {
                    write!(s, "{:?}: {}, ", hint, count)?;
                }
            }
            write!(
                f,
                "PerftData {{ move_count: {{ {}}}, check_count: {}, mate_count: {} }}",
                s, self.check_count, self.checkmate_count
            )
        }
    }
}

use crate::game::move_generator::*;

use crate::game::move_generator::position::*;

use data::PerftData;

pub fn perft_leaves(mut position: Position, depth: u8) -> PerftData {
    let mut data = PerftData::new();
    collect_perft(
        &mut position,
        &mut MoveGenerator::empty(),
        &mut data,
        depth - 1,
    );
    return data;
}

pub fn count_leaves(mut position: Position, depth: u8) -> u128 {
    //return count_legal_moves(&mut position, &mut MoveGenerator::new(), depth - 1)
    return count_legal_moves(&mut position, &mut MoveGenerator::empty(), depth - 1);
    //return count_legal_moves_game(&mut Game::from_position(position), depth - 1);
}

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

// use std::fmt;
//
// impl fmt::Display for PerftValues {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         writeln!(f, "{:=^169}", "")?;
//         writeln!(
//             f,
//             "={:^30}={:^20}={:^20}={:^20}={:^20}={:^20}={:^20}=",
//             "NODES", "CAPTURES", "EP", "CASTLES", "PROMOTIONS", "CHECKS", "CHECKMATES"
//         )?;
//         writeln!(
//             f,
//             "={:^30}={:^20}={:^20}={:^20}={:^20}={:^20}={:^20}=",
//             self.nodes,
//             self.captures,
//             self.ep,
//             self.castles,
//             self.promotions,
//             self.checks,
//             self.checkmates
//         )?;
//         writeln!(f, "{:=^169}", "")?;
//         Ok(())
//     }
// }

fn can_move(game_state: &mut Position, move_list: &mut MoveGenerator) -> bool {
    move_list.generate_moves(game_state);

    while let Some(next_move) = move_list.pop_move() {
        let unmove = make_move(game_state, next_move);

        if !game_state.board().is_king_in_check(!game_state.turn()) {
            unmake_move(game_state, unmove);
            move_list.pop_group();
            return true;
        }

        unmake_move(game_state, unmove);
    }

    move_list.pop_group();
    return false;
}

fn count_legal_moves(game_state: &mut Position, move_list: &mut MoveGenerator, depth: u8) -> u128 {
    let mut count = 0u128;
    move_list.generate_moves(game_state);

    while let Some(next_move) = move_list.pop_move() {
        let unmove = make_move(game_state, next_move);

        if !game_state.board().is_king_in_check(!game_state.turn()) {
            if depth == 0 {
                count += 1;
            } else {
                count += count_legal_moves(game_state, move_list, depth - 1);
            }
        }

        unmake_move(game_state, unmove);
    }

    move_list.pop_group();
    return count;
}

// fn count_legal_moves_game(game: &mut Game, depth: u8) -> u128 {
//     let mut count = 0u128;

//     game.for_each_legal_child_node(
//         |node| {
//             if depth == 0 { count += 1; }
//             else { count += count_legal_moves_game(node, depth - 1); }
//         }
//     );

//     return count;
// }

// fn count_generator_legal_moves(move_generator: &mut MoveGenerator, depth: u8) -> u128 {
//     let mut count = 0u128;
//     if move_generator.inspect_child_nodes() != NodeStatus::Internal {
//         return 0;
//     }

//     if depth == 0 {
//         loop {
//             count += 1;
//             if !move_generator.to_next_child_node() { break; }
//         }
//     }
//     else {
//         loop {
//             count += count_generator_legal_moves(move_generator, depth - 1);
//             if !move_generator.to_next_child_node() { break; }
//         }
//     }

//     return count;
// }

fn collect_perft(
    game_state: &mut Position,
    move_list: &mut MoveGenerator,
    data: &mut PerftData,
    depth: u8,
) {
    move_list.generate_moves(game_state);

    while let Some(next_move) = move_list.pop_move() {
        let unmove = make_move(game_state, next_move);

        if !game_state.board().is_king_in_check(!game_state.turn()) {
            if depth == 0 {
                data.add_move(next_move.hint());
                if game_state.board().is_king_in_check(game_state.turn()) {
                    data.add_check();
                    if !can_move(game_state, move_list) {
                        data.add_checkmate();
                    }
                }
            } else {
                collect_perft(game_state, move_list, data, depth - 1);
            }
        }

        unmake_move(game_state, unmove);
    }

    move_list.pop_group();
}
