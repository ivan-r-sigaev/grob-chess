use std::{sync::Arc, thread, time::Instant};

use crossbeam::channel::{Receiver, Sender};
use either::Either;

use crate::{
    search::{
        scheduler::SearchResult,
        signals::{WorkerSignaler, WorkerSignalerMaster},
        transposition::{Transposition, TranspositionTable},
    },
    GameEnding, GameExplorer, MoveOrdering, Piece, Score, SearchRequest, ServerResponse,
};

#[derive(Debug, Clone)]
pub struct Job {
    pub request: SearchRequest,
    pub batch_index: usize,
}

#[derive(Debug)]
pub struct WorkerGroup {
    signaler: Option<WorkerSignalerMaster>,
    job_recv: Receiver<Job>,
    res_send: Sender<ServerResponse>,
    tt: Arc<TranspositionTable>,
}

impl WorkerGroup {
    pub fn new(
        worker_count: usize,
        job_recv: Receiver<Job>,
        res_send: Sender<ServerResponse>,
        tt: Arc<TranspositionTable>,
    ) -> Self {
        let mut res = Self {
            signaler: None,
            job_recv,
            res_send,
            tt,
        };
        res.spawn_workers(worker_count);
        res
    }
    pub fn signaler(&self) -> &WorkerSignalerMaster {
        self.signaler.as_ref().unwrap()
    }
    pub fn resize(&mut self, new_worker_count: usize) {
        if new_worker_count == self.signaler().worker_count() {
            return;
        }

        self.clear();
        self.spawn_workers(new_worker_count);
    }
    fn clear(&mut self) {
        if let Some(signaler) = self.signaler.take() {
            signaler.quit();
        }
    }
    fn spawn_workers(&mut self, worker_count: usize) {
        assert!(self.signaler.is_none());
        let mut master = WorkerSignalerMaster::new(worker_count);

        for _ in 0..worker_count {
            let signaler = master.create_signaler();
            let job_recv = self.job_recv.clone();
            let res_send = self.res_send.clone();
            let tt = self.tt.clone();
            thread::spawn(|| {
                Worker {
                    signaler,
                    job_recv,
                    res_send,
                    tt,
                }
                .run()
            });
        }

        self.signaler = Some(master);
    }
}

impl Drop for WorkerGroup {
    fn drop(&mut self) {
        self.clear();
    }
}

#[derive(Debug, Clone, Copy)]
struct SearchConstraints {
    pub nodes_max: Option<u64>,
    pub deadline: Option<Instant>,
}

impl SearchConstraints {
    pub fn nodes_fail(self, nodes: u64) -> bool {
        self.nodes_max.is_some_and(|max| nodes > max)
    }
    pub fn time_fails(self) -> bool {
        self.deadline.is_some_and(|d| Instant::now() > d)
    }
}

#[derive(Debug)]
struct Worker {
    signaler: WorkerSignaler,
    job_recv: Receiver<Job>,
    res_send: Sender<ServerResponse>,
    tt: Arc<TranspositionTable>,
}

impl Worker {
    fn run(&mut self) {
        loop {
            self.signaler.wakeup();

            while let Ok(job) = self.job_recv.try_recv() {
                let mut game = job.request.game;
                let worst_score = Score::ending(GameEnding::Checkmate);
                let result = self.search(
                    &mut game.explore(),
                    job.request.depth,
                    SearchConstraints {
                        nodes_max: job.request.nodes,
                        deadline: job.request.deadline,
                    },
                    worst_score,
                    worst_score.prev(),
                );
                let result = ServerResponse {
                    result,
                    batch_index: job.batch_index,
                };
                self.res_send.send(result).unwrap();
            }

            if self.signaler.should_quit() {
                break;
            }

            self.signaler.sleep();
        }
    }
    fn search(
        &mut self,
        node: &mut GameExplorer,
        depth: u64,
        constraints: SearchConstraints,
        mut alpha: Score,
        beta: Score,
    ) -> SearchResult {
        if self.signaler.should_stop() || constraints.time_fails() {
            return self.evaluate(node, true);
        }

        let position = node.game();
        let hash = position.zobrist();
        if let Some(t) = self.tt.get(hash) {
            'probe_hash: {
                if position.is_move_pseudo_legal(t.best_move) {
                    break 'probe_hash;
                }
                if t.depth < depth {
                    // TODO: should optimize for hash move here.
                    break 'probe_hash;
                }
                return SearchResult {
                    best_move: Some(t.best_move),
                    score: t.score,
                    nodes: 1,
                    is_canceled: false,
                };
            }
        }

        if depth == 0 {
            return self.quiescence(node, alpha, beta);
        }

        let mut best_move = None;
        let mut best_score = None;
        let mut nodes = 1;
        let mut is_canceled = false;
        let maybe_ending =
            node.for_each_legal_child_node(MoveOrdering::MvvLva, |node, chess_move| {
                let result = self.search(node, depth - 1, constraints, alpha, beta);
                nodes += result.nodes;
                if best_score.is_none_or(|score| result.score.prev() > score) {
                    best_score = Some(result.score);
                    best_move = Some(chess_move);
                }

                if result.score > alpha {
                    alpha = result.score;
                }

                if result.score >= beta {
                    node.exhaust_moves();
                    return;
                }

                if constraints.nodes_fail(nodes)
                    || constraints.time_fails()
                    || self.signaler.should_stop()
                {
                    is_canceled = true;
                    node.exhaust_moves();
                }
            });
        let score = match maybe_ending {
            Some(ending) => Score::ending(ending),
            None => {
                let best_move = best_move.unwrap();
                let score = best_score.unwrap();
                self.tt.insert(
                    hash,
                    Transposition {
                        best_move,
                        depth,
                        score,
                    },
                );
                score
            }
        };

        SearchResult {
            best_move,
            score,
            nodes,
            is_canceled,
        }
    }
    fn quiescence(&mut self, node: &mut GameExplorer, alpha: Score, beta: Score) -> SearchResult {
        if self.signaler.should_stop() {
            return self.evaluate(node, true);
        }

        _ = (alpha, beta);

        // TODO: implement quiescence search.
        self.evaluate(node, false)
    }
    fn evaluate(&mut self, node: &mut GameExplorer, is_canceled: bool) -> SearchResult {
        let nodes = 1;
        let any_move = match node.check_ending() {
            Either::Left(chess_move) => chess_move,
            Either::Right(ending) => {
                return SearchResult {
                    best_move: None,
                    score: Score::ending(ending),
                    nodes,
                    is_canceled,
                }
            }
        };

        let position = node.game();
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
        let p = p_score + (n_score + b_score) * 3 + r_score * 5 + q_score * 9;
        let score = Score::Cp(p * 100);

        SearchResult {
            best_move: Some(any_move),
            score,
            nodes,
            is_canceled,
        }
    }
}
