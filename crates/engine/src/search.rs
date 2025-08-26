use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use crossbeam::channel::{Receiver, RecvError, Select, Sender, TryRecvError};
use game::{
    ChessMove, Color, Game, LanMove, MoveOrdering, Score, SearchRequest, ServerCommand,
    ServerResponse, spawn_search_server,
};

use crate::uci::Go;

#[derive(Debug, Clone, Copy)]
pub struct SearchResult {
    pub best_move: Option<LanMove>,
    pub ponder: Option<LanMove>,
}

#[derive(Debug)]
pub struct Search {
    server_send: Sender<ServerCommand>,
    server_recv: Receiver<ServerResponse>,
    progress: Option<SearchProgress>,
}

#[derive(Debug, Clone)]
struct SearchProgress {
    pub game: Game,
    pub moves: HashMap<ChessMove, Option<game::SearchResult>>,
    pub deadline: Option<Instant>,
    pub nodes_max: Option<u64>,
    pub depth_max: Option<u64>,
    pub mate: Option<u64>,
    pub pending_count: usize,
    pub running_depth: u64,
    pub ponder: bool,
    pub infinite: bool,
    pub pending_result: Option<SearchResult>,
}

impl Search {
    pub fn new() -> Self {
        const WORKER_COUNT: usize = 1;
        const TT_MAX_MIB: usize = 16;
        let (server_send, server_recv) = spawn_search_server(WORKER_COUNT, TT_MAX_MIB);
        Self {
            server_send,
            server_recv,
            progress: None,
        }
    }
    pub fn is_running(&self) -> bool {
        self.progress.is_some()
    }
    pub fn add_to_select<'a>(&'a self, sel: &mut Select<'a>) -> usize {
        if !self.is_running() {
            panic!(concat!(
                "Do not try to wait on the search sever ",
                "unless there is an ongoing search",
            ));
        }
        sel.recv(&self.server_recv)
    }
    pub fn go(&mut self, mut game: Game, go: Go) {
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
                game.explore()
                    .for_each_legal_child_node(MoveOrdering::MvvLva, |_, chess_move| {
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
        let pending_result = game.explore().check_ending().right().map(|_| SearchResult {
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
            pending_count: 0,
            running_depth: 0,
            pending_result,
        });
        if pending_result.is_some() {
            return;
        }

        self.prepare();
    }
    pub fn stop(&mut self) -> SearchResult {
        if let Some(pending) = self.get_progress().pending_result {
            self.progress = None;
            return pending;
        }

        self.server_send.send(ServerCommand::Cancel).unwrap();
        self.recv();
        let (best_move, _, _) = self.collect();
        self.progress = None;
        SearchResult {
            best_move: Some(best_move.lan()),
            ponder: None,
        }
    }
    pub fn check(&mut self) -> Option<SearchResult> {
        if !self.try_recv() {
            return None;
        }

        let (best_move, should_stop, should_hold) = self.collect();
        let progress = self.get_progress();

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

        self.server_send.send(ServerCommand::ClearHash).unwrap();
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
    fn collect(&mut self) -> (ChessMove, bool, bool) {
        let progress = self.get_progress();
        let mut best_move = None;
        let mut score = None;
        let mut nodes = 0;
        let mut unfinished = false;

        for (&chess_move, &result) in progress.moves.iter() {
            let result = result.unwrap();
            if score.is_none_or(|s| result.score > s) {
                score = Some(result.score);
                best_move = Some(chess_move);
            }
            nodes += result.nodes;
            unfinished |= result.is_canceled;
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
        assert!(self.get_progress().pending_count == 0);
        let progress = self.progress.as_mut().unwrap();
        progress.pending_count = progress.moves.len();
        let mut vec = Vec::with_capacity(progress.pending_count);
        for &chess_move in progress.moves.keys() {
            let mut game = progress.game.clone();
            game.make_move(chess_move);
            vec.push(SearchRequest {
                game,
                depth: progress.running_depth,
                nodes: progress.nodes_max,
                deadline: progress.deadline,
            });
        }
        self.server_send
            .send(ServerCommand::ProcessBatch(vec))
            .unwrap();
    }
    fn recv(&mut self) {
        while self.get_progress().pending_count > 0 {
            let response = match self.server_recv.recv() {
                Ok(response) => response,
                Err(RecvError) => panic!("Search server disconnected!"),
            };
            self.update(response);
            self.get_progress().pending_count -= 1;
        }
    }
    fn try_recv(&mut self) -> bool {
        while self.get_progress().pending_count > 0 {
            let response = match self.server_recv.try_recv() {
                Ok(response) => response,
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => panic!("Search server disconnected!"),
            };
            self.update(response);
            self.get_progress().pending_count -= 1;
        }

        self.get_progress().pending_count == 0
    }
    fn update(&mut self, response: ServerResponse) {
        let key = *self
            .get_progress()
            .moves
            .keys()
            .nth(response.batch_index)
            .unwrap();
        let maybe_result = self.get_progress().moves.get_mut(&key).unwrap();
        let result = maybe_result.get_or_insert(response.result);
        if !response.result.is_canceled {
            *result = response.result;
        }
    }
    fn get_progress(&mut self) -> &mut SearchProgress {
        self.progress.as_mut().unwrap()
    }
}
