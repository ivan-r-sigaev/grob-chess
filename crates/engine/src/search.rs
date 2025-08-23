use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use board::Color;
use game::{ParallelSearch, Score, Transposition, Waiter};
use position::{ChessMove, LanMove, Position};

use crate::uci::Go;

#[derive(Debug, Clone, Copy)]
pub struct SearchResult {
    pub best_move: Option<LanMove>,
    pub ponder: Option<LanMove>,
}

#[derive(Debug)]
pub struct Search {
    search: ParallelSearch,
    progress: Option<SearchProgress>,
}

#[derive(Debug, Clone)]
struct SearchProgress {
    pub game: Position,
    pub moves: HashMap<ChessMove, Option<game::SearchResult>>,
    pub deadline: Option<Instant>,
    pub nodes_max: Option<u64>,
    pub depth_max: Option<u64>,
    pub mate: Option<u64>,
    pub ponder: bool,
    pub infinite: bool,
    pub running_depth: u64,
    pub pending_result: Option<SearchResult>,
}

impl Search {
    pub fn new() -> Self {
        const THREAD_COUNT: usize = 1;
        const TT_CAPACITY_IN_BYTES: usize = 16 * 1024 * 1024;
        const TT_CAPACITY: usize = TT_CAPACITY_IN_BYTES / size_of::<Transposition>();
        let search = ParallelSearch::new(THREAD_COUNT, TT_CAPACITY);
        Self {
            search,
            progress: None,
        }
    }
    pub fn is_running(&self) -> bool {
        self.progress.is_some()
    }
    pub fn add_to_waiter<'a>(&'a self, waiter: &mut Waiter<'a>) -> usize {
        self.search.add_to_waiter(waiter, None)
    }
    pub fn go(&mut self, mut game: Position, go: Go) {
        if self.is_running() {
            panic!(concat!(
                "Do not send the go command to the Search ",
                "until the ongoing search is finished.",
            ));
        }

        let moves = go
            .searchmoves
            .map(|moves| {
                let mut vec = Vec::new();
                for lan_move in moves {
                    let Some(chess_move) = game.lan_move(lan_move) else {
                        continue;
                    };
                    vec.push(chess_move);
                }
                vec
            })
            .filter(|vec| !vec.is_empty())
            .unwrap_or({
                let mut vec = Vec::new();
                game.search().for_each_legal_child_node(|_, chess_move| {
                    vec.push(chess_move);
                });
                vec
            });
        let moves = {
            let mut map = HashMap::new();
            for chess_move in moves {
                map.insert(chess_move, None);
            }
            map
        };
        let deadline = go
            .movetime
            .or_else(|| {
                let turn = game.turn();
                let inc = match turn {
                    Color::White => go.winc,
                    Color::Black => go.binc,
                }
                .unwrap_or(Duration::ZERO);
                match turn {
                    Color::White => go.wtime,
                    Color::Black => go.btime,
                }
                .map(|time| time + inc)
            })
            .map(|d| Instant::now() + d);
        _ = go.movestogo;
        let nodes_max = go.nodes;
        let depth_max = go.depth.map(|d| d - 1);
        let mate = go.mate;
        let ponder = go.ponder;
        let infinite = go.infinite;
        let pending_result = game.search().check_ending().right().map(|_| SearchResult {
            best_move: None,
            ponder: None,
        });
        self.progress = Some(SearchProgress {
            game,
            moves,
            deadline,
            nodes_max,
            depth_max,
            mate,
            ponder,
            infinite,
            running_depth: 0,
            pending_result,
        });
        if pending_result.is_some() {
            return;
        }

        self.prepare();
    }
    pub fn stop(&mut self) -> SearchResult {
        let Some(ref mut progress) = self.progress else {
            panic!(concat!(
                "Do not use the check method ",
                "unless there is an ongoing search.",
            ));
        };
        if let Some(pending) = progress.pending_result {
            self.progress = None;
            return pending;
        }

        let results = self.search.stop();
        let (best_move, _, _) = Self::collect(progress, results);
        self.progress = None;
        SearchResult {
            best_move: Some(best_move.lan()),
            ponder: None,
        }
    }
    pub fn check(&mut self) -> Option<SearchResult> {
        let Some(ref mut progress) = self.progress else {
            panic!(concat!(
                "Do not use the check method ",
                "unless there is an ongoing search.",
            ));
        };
        let results = self.search.try_collect()?;

        let (best_move, should_stop, should_hold) = Self::collect(progress, results);

        if should_stop {
            let result = Some(SearchResult {
                best_move: Some(best_move.lan()),
                ponder: None,
            });
            if should_hold {
                progress.pending_result = result;
            } else {
                self.progress = None;
                return result;
            }
        } else {
            progress.running_depth += 1;
            self.prepare();
        }

        None
    }
    pub fn clear_tt(&mut self) {
        if self.is_running() {
            panic!(concat!(
                "Do not try to clear the transposition table ",
                "until the ongoing search is finished.",
            ));
        }

        self.search.clear_tt();
    }
    pub fn ponderhit(&mut self) {
        let Some(ref mut progress) = self.progress else {
            panic!(concat!(
                "Do not use the ponderhit method ",
                "unless there is an ongoing search.",
            ));
        };

        progress.ponder = false;
    }
    fn collect(
        progress: &mut SearchProgress,
        results: Vec<game::SearchResult>,
    ) -> (ChessMove, bool, bool) {
        let mut best_move = None;
        let mut score = None;
        let mut nodes = 0;
        let mut unfinished = false;

        for (i, (&chess_move, &fallback)) in progress.moves.iter().enumerate() {
            let recent = results[i];
            let result = match fallback {
                Some(res) if recent.unfinished => res,
                None | Some(_) => recent,
            };

            if score.is_none_or(|s| result.score > s) {
                score = Some(result.score);
                best_move = Some(chess_move);
            }
            nodes += result.nodes;
            unfinished |= result.unfinished;
        }

        let best_move = best_move.unwrap();
        let score = score.unwrap();

        let depth_fail = progress
            .depth_max
            .is_some_and(|max| progress.running_depth >= max);
        let nodes_fail = progress.nodes_max.is_some_and(|max| nodes >= max);
        let mate_fail = progress
            .mate
            .is_some_and(|n| score >= Score::Mating(n) || score <= Score::Mated(n));
        let should_stop = unfinished | depth_fail | nodes_fail | mate_fail;
        let only_depth = depth_fail & !nodes_fail & !mate_fail;
        let should_hold = should_stop & (progress.ponder | (only_depth & progress.infinite));

        (best_move, should_stop, should_hold)
    }
    fn prepare(&mut self) {
        let progress = self.progress.as_mut().unwrap();
        for &chess_move in progress.moves.keys() {
            let mut game = progress.game.clone();
            assert!(game.try_make_move(chess_move));
            self.search.prepare_search(
                game,
                progress.running_depth,
                progress.nodes_max,
                progress.deadline,
            );
        }
        self.search.go();
    }
}
