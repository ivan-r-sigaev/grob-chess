use crate::game::{GameEnding, GameSearch};
use crate::transposition_table::{Entry, TranspositionTable};
use position::board::Piece;
use position::position::ChessMove;

pub fn evaluate<const TT_SIZE: usize>(
    node: &mut GameSearch,
    depth: u8,
) -> (i32, Option<ChessMove>) {
    let mut tt = TranspositionTable::<TT_SIZE, ChessMove>::new();
    negamax(&mut tt, node, -100, 100, depth)
}

fn negamax<const TT_SIZE: usize>(
    tt: &mut TranspositionTable<TT_SIZE, ChessMove>,
    node: &mut GameSearch,
    mut alpha: i32,
    beta: i32,
    depth: u8,
) -> (i32, Option<ChessMove>) {
    if depth == 0 {
        return (evaluate_static(node), None);
    }
    let hash = node.get().position().position_hash();
    if let Ok(Entry::Occupied(occupied)) = tt.entry(hash) {
        let &chess_move = occupied.get();
        if node.map_move_if_legal(chess_move, |node| {
            alpha = beta.min(-negamax(tt, node, -beta, -alpha, depth - 1).0);
        }) {
            return (alpha, Some(chess_move));
        }
    }
    let mut best_move: Option<ChessMove> = None;
    let score = match node.for_each_legal_child_node(|node, move_concept| {
        let score = -negamax(tt, node, -beta, -alpha, depth - 1).0;
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
                tt.entry(hash)
                    .unwrap_or_else(|collision| Entry::Occupied(collision.keep_key()))
                    .insert_entry(move_concept);
            }
            alpha
        }
        Some(ending) => evaluate_ending(ending),
    };
    (score, best_move)
}

fn evaluate_static(node: &mut GameSearch<'_>) -> i32 {
    if let Some(ending) = node.check_ending() {
        return evaluate_ending(ending);
    }

    let position = node.get().position();
    let board = position.board();
    let player = board.get_color(position.turn());
    let queens = board.get_piece(Piece::Queen);
    let rooks = board.get_piece(Piece::Rook);
    let bishops = board.get_piece(Piece::Bishop);
    let knights = board.get_piece(Piece::Knight);
    let pawns = board.get_piece(Piece::Pawn);
    let q_score = (queens & player).count() as i32 - (queens & !player).count() as i32;
    let r_score = (rooks & player).count() as i32 - (rooks & !player).count() as i32;
    let b_score = (bishops & player).count() as i32 - (bishops & !player).count() as i32;
    let n_score = (knights & player).count() as i32 - (knights & !player).count() as i32;
    let p_score = (pawns & player).count() as i32 - (pawns & !player).count() as i32;

    p_score + (n_score + b_score) * 3 + r_score * 5 + q_score * 9
}

fn evaluate_ending(ending: GameEnding) -> i32 {
    match ending {
        GameEnding::Checkmate => -100,
        GameEnding::Stalemate => 0,
    }
}
