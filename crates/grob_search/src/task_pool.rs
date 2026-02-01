// A: *Needs to search moves a, b, c.*
// A: Allocate tasks for a, b, c (using worker).
// A: Allocate result r, link a, b, c to r.
// A: Push a, b, c to queue.
// A, B, C: process queue.
// A: await r.

use std::sync::{Arc, atomic::{AtomicI32, AtomicUsize}};

use crossbeam::{deque, utils::CachePadded};
use grob_core::game::{Game, movegen::ChessMove};
use parking_lot::Mutex;

use crate::pv::PvTable;

#[derive(Debug)]
pub struct Task {
    chess_move: ChessMove,
    result: Arc<TaskResult>,
}

impl Task {
    pub fn game(&self) -> Game {
        let mut game = self.result.game;
        game.make_move(self.chess_move);
        game
    }
}

#[derive(Debug)]
pub struct TaskBuilder<'worker> {
    worker: &'worker mut Worker,
}

#[derive(Debug)]
pub struct Worker {
    tasks: deque::Worker<Task>,
    staged_tasks: Vec<Task>,
    results: Vec<Arc<TaskResult>>,
    results_used_count: usize,
}

#[derive(Debug)]
struct TaskResult {
    countdown: CachePadded<AtomicUsize>,
    alpha: CachePadded<AtomicI32>,
    beta: i32,
    game: Game,
    ply: usize,
    pv: Option<Arc<Mutex<PvTable>>>,
}
