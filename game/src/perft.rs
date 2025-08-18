use position::position::ChessMoveHint;

use crate::game::GameSearch;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PerftValues {
    pub nodes: u64,
    pub captures: u64,
    pub ep: u64,
    pub castles: u64,
    pub promotions: u64,
    pub checks: u64,
    pub checkmates: u64,
}

impl PerftValues {
    pub fn collect(node: &mut GameSearch, depth: u8) -> Self {
        let mut values = PerftValues::empty();
        values.search(node, depth);
        values
    }
    pub fn empty() -> Self {
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
    fn search(&mut self, node: &mut GameSearch, depth: u8) {
        _ = node.for_each_legal_child_node(|node, chess_move| {
            if depth != 0 {
                self.search(node, depth - 1);
                return;
            }
            self.add_move(chess_move.hint());

            if !node.game().position().is_check() {
                return;
            }
            self.add_check();

            if !node.check_ending().is_right() {
                return;
            }
            self.add_checkmate();
        })
    }
}

#[cfg(test)]
mod tests;
