use crate::game::{Game, GameEnding};
use crate::transposition_table::TranspositionTable;
use position::prelude::ChessMove;

pub fn evaluate<const TT_SIZE: usize>(game: &mut Game, depth: u8) -> i32 {
    let mut tt = TranspositionTable::<TT_SIZE, ChessMove>::new();
    negamax(&mut tt, game, -100, 100, depth)
}

fn negamax<const TT_SIZE: usize>(
    tt: &mut TranspositionTable<TT_SIZE, ChessMove>,
    game: &mut Game,
    mut alpha: i32,
    beta: i32,
    depth: u8,
) -> i32 {
    if depth == 0 {
        return 0;
    }
    let hash = game.get_position().position_hash();
    if let Some(move_concept) = tt.get(hash) {
        if game.map_move_if_legal(*move_concept, |node, _| {
            alpha = beta.min(-negamax(tt, node, -beta, -alpha, depth - 1));
        }) {
            return alpha;
        }
    }
    let mut best_move: Option<ChessMove> = None;
    match game.for_each_legal_child_node(|node, move_concept| {
        let score = -negamax(tt, node, -beta, -alpha, depth - 1);
        if score >= alpha {
            best_move = Some(move_concept);
            alpha = score;
        }
        if alpha >= beta {
            alpha = beta;
            node.exhaust_moves();
        }
    }) {
        None => {
            if let Some(move_concept) = best_move {
                tt.insert(hash, move_concept);
            }
            alpha
        }
        Some(GameEnding::Checkmate) => -100,
        Some(GameEnding::Stalemate) => 0,
    }
}

// use crate::hash_table::*;
// use crate::position::*;

// impl MoveGenerator {
// pub fn evaluate(&mut self, mut alpha: i32, beta: i32, depth: u8) -> i32 {
//     if depth == 0 { return self.get_static_evaluation_score(); }
//     match self.inspect_child_nodes() {
//         None => {},
//         Some(GameEnding::Checkmate) => return -100,
//         Some(GameEnding::Stalemate) => return 0,
//     };

//     loop {
//         alpha = alpha.max(-self.evaluate(-beta, -alpha, depth - 1));
//         if alpha >= beta {
//             self.to_parent_node();
//             return beta;
//         }
//         if !self.to_next_child_node() { break; }
//     }

//     return alpha;
// }
//     pub fn get_static_evaluation_score(&mut self) -> i32 {
//         return 0;
//     }
// }
