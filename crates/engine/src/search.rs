use std::{
    thread,
    time::{Duration, Instant},
};

use crossbeam::{
    channel::{Receiver, RecvError, SendError, Sender, unbounded},
    select,
};
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

#[derive(Debug, Clone)]
pub enum SearchCommand {
    Go(Box<Go>, Game),
    Stop,
    PonderHit,
    UciNewGame,
}

pub fn spawn_uci_server() -> (Sender<SearchCommand>, Receiver<SearchResult>) {
    let (search_send, r) = unbounded();
    let (s, search_recv) = unbounded();
    thread::spawn(move || UciServer::new(search_send, search_recv).run());
    (s, r)
}

#[derive(Debug)]
struct UciServer {
    server_send: Sender<ServerCommand>,
    server_recv: Receiver<ServerResponse>,
    search_send: Sender<SearchResult>,
    search_recv: Receiver<SearchCommand>,
    progress: Option<SearchProgress>,
}

/// Limits of the search.
#[derive(Debug, Clone, Copy)]
struct SearchLimits {
    /// Search no further than this depth.
    depth: Option<u64>,
    /// Search no more than this many nodes.
    nodes: Option<u64>,
    /// Stop search if mate in less than this many turns is found.
    mate: Option<u64>,
    /// Stop search when reaching the deadline.
    deadline: Option<Instant>,
}

/// State of the ongoing search.
#[derive(Debug, Clone)]
struct SearchProgress {
    /// Position that is being explored (either the searched one
    /// or a suggested ponder move).
    game: Game,
    /// Sorted vector of the searched moves.
    moves: Vec<(ChessMove, Option<game::SearchResult>)>,
    /// Limits of the search.
    limits: SearchLimits,
    /// Current iterative deepening depth.
    running_depth: u64,
    /// Number of remaining search results to recieve before
    /// increasing the current depth.
    pending_count: usize,
    /// Whether the search is currently running in pondering mode.
    is_pondering: bool,
}

/// Simple utility type to improve control flow.
#[derive(Debug)]
struct ShouldQuit;

type Result = std::result::Result<(), ShouldQuit>;

impl UciServer {
    pub fn new(search_send: Sender<SearchResult>, search_recv: Receiver<SearchCommand>) -> Self {
        let worker_count = 1;
        let tt_max_capacity_mib = 16;
        let (server_send, server_recv) = spawn_search_server(worker_count, tt_max_capacity_mib);
        Self {
            server_send,
            server_recv,
            search_send,
            search_recv,
            progress: None,
        }
    }
    pub fn run(&mut self) {
        loop {
            if let Err(ShouldQuit) = self.run_inner() {
                break;
            }
        }
    }
    fn run_inner(&mut self) -> Result {
        select! {
            recv(&self.search_recv) -> result => {
                let command = result.map_err(|RecvError| ShouldQuit)?;
                self.handle_command(command)
            }
            recv(&self.server_recv) -> result => self.update(result.unwrap()),
        }
    }
    fn is_holding(&self) -> bool {
        self.progress.is_some() && self.progress().pending_count == 0
    }
    fn handle_command(&mut self, command: SearchCommand) -> Result {
        match command {
            SearchCommand::Go(go, game) => self.go(*go, game),
            SearchCommand::Stop => self.stop(),
            SearchCommand::PonderHit => self.ponderhit(),
            SearchCommand::UciNewGame => self.ucinewgame(),
        }
    }
    fn go(&mut self, go: Go, mut game: Game) -> Result {
        assert!(self.progress.is_none());

        let moves = go
            .searchmoves
            .map(|moves| {
                // TODO: sort the vec...
                let mut vec = Vec::new();
                for lan_move in moves {
                    let Some(chess_move) = game.lan_move(lan_move) else {
                        continue;
                    };
                    vec.push((chess_move, None));
                }
                vec
            })
            .filter(|vec| !vec.is_empty())
            .unwrap_or({
                let mut vec = Vec::new();
                game.explore()
                    .for_each_legal_child_node(MoveOrdering::MvvLva, |_, chess_move| {
                        vec.push((chess_move, None));
                    });
                vec
            });
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
        let depth_max = go.depth.map(|d| d - 1).filter(|_| !go.infinite);
        let mate = go.mate;
        let ponder = go.ponder;
        self.progress = Some(SearchProgress {
            game,
            moves,
            limits: SearchLimits {
                depth: depth_max,
                nodes: nodes_max,
                mate,
                deadline,
            },
            is_pondering: ponder,
            pending_count: 0,
            running_depth: 0,
        });
        if self.progress_mut().game.explore().check_ending().is_right() {
            return Ok(());
        }

        self.prepare();
        Ok(())
    }
    fn stop(&mut self) -> Result {
        if self.progress.is_none() {
            return Ok(());
        }

        self.server_send.send(ServerCommand::Cancel).unwrap();
        while self.progress().pending_count != 0 {
            self.update(self.server_recv.recv().unwrap())?;
        }

        self.check(true)
    }
    fn ponderhit(&mut self) -> Result {
        if self.progress.is_none() {
            return Ok(());
        }

        self.progress_mut().is_pondering = false;
        if self.is_holding() {
            self.check(false)?;
        }

        Ok(())
    }
    fn ucinewgame(&mut self) -> Result {
        self.server_send.send(ServerCommand::ClearHash).unwrap();
        Ok(())
    }
    fn update(&mut self, rsp: ServerResponse) -> Result {
        assert!(self.progress().pending_count > 0);
        let result = self
            .progress_mut()
            .moves
            .iter_mut()
            .map(|(_, value)| value)
            .nth(rsp.batch_index)
            .unwrap()
            .get_or_insert(rsp.result);
        if !rsp.result.is_canceled {
            *result = rsp.result;
        }
        self.progress_mut().pending_count -= 1;
        self.check(false)
    }
    fn check(&mut self, force: bool) -> Result {
        if self.progress().pending_count != 0 {
            return Ok(());
        }

        let (best_move, should_stop, should_hold) = self.collect();

        if should_stop || force {
            let result = SearchResult {
                best_move: best_move.map(|m| m.lan()),
                ponder: None,
            };
            if !should_hold || force {
                self.progress = None;
                self.search_send
                    .send(result)
                    .map_err(|SendError(_)| ShouldQuit)?;
            }
            return Ok(());
        }

        self.progress_mut().running_depth += 1;
        self.prepare();

        Ok(())
    }
    fn collect(&mut self) -> (Option<ChessMove>, bool, bool) {
        assert!(self.progress().pending_count == 0);
        let progress = self.progress();
        let mut best_move = None;
        let mut score = None;
        let mut nodes = 0;
        let mut unfinished = false;

        for &(chess_move, result) in progress.moves.iter() {
            let result = result.unwrap();
            if score.is_none_or(|s| result.score > s) {
                score = Some(result.score);
                best_move = Some(chess_move);
            }
            nodes += result.nodes;
            unfinished |= result.is_canceled;
        }

        if best_move.is_none() {
            return (None, true, progress.is_pondering);
        }

        let best_move = best_move.unwrap();
        let score = score.unwrap();

        let time_fails = progress
            .limits
            .deadline
            .is_some_and(|d| Instant::now() >= d);
        let depth_fails = progress
            .limits
            .depth
            .is_some_and(|max| progress.running_depth >= max);
        let nodes_fail = progress.limits.nodes.is_some_and(|max| nodes >= max);
        let mate_fail = progress
            .limits
            .mate
            .is_some_and(|n| score >= Score::Mating(n) || score <= Score::Mated(n));
        let depth_limited = progress.running_depth == u8::MAX as u64;
        let should_stop =
            unfinished | time_fails | depth_fails | nodes_fail | mate_fail | depth_limited;
        let should_hold = should_stop & (progress.is_pondering | depth_limited);

        (Some(best_move), should_stop, should_hold)
    }
    fn prepare(&mut self) {
        assert!(self.progress().pending_count == 0);
        let progress = self.progress.as_mut().unwrap();
        progress.pending_count = progress.moves.len();
        let mut vec = Vec::with_capacity(progress.pending_count);
        for &(chess_move, _) in &progress.moves {
            let mut game = progress.game.clone();
            game.make_move(chess_move);
            vec.push(SearchRequest {
                game,
                depth: progress.running_depth,
                nodes: progress.limits.nodes,
                deadline: progress.limits.deadline,
            });
        }
        self.server_send
            .send(ServerCommand::ProcessBatch(vec))
            .unwrap();
    }
    fn progress(&self) -> &SearchProgress {
        self.progress.as_ref().unwrap()
    }
    fn progress_mut(&mut self) -> &mut SearchProgress {
        self.progress.as_mut().unwrap()
    }
}

// #[derive(Debug)]
// pub struct Search {
//     server_send: Sender<ServerCommand>,
//     server_recv: Receiver<ServerResponse>,
//     progress: Option<SearchProgress>,
// }

// impl Search {
//     pub fn new() -> Self {
//         const WORKER_COUNT: usize = 1;
//         const TT_MAX_MIB: usize = 16;
//         let (server_send, server_recv) = spawn_search_server(WORKER_COUNT, TT_MAX_MIB);
//         Self {
//             server_send,
//             server_recv,
//             progress: None,
//         }
//     }
//     pub fn is_running(&self) -> bool {
//         self.progress.is_some()
//     }
//     pub fn add_to_select<'a>(&'a self, sel: &mut Select<'a>) -> usize {
//         if !self.is_running() {
//             panic!(concat!(
//                 "Do not try to wait on the search sever ",
//                 "unless there is an ongoing search",
//             ));
//         }
//         sel.recv(&self.server_recv)
//     }
//     pub fn go(&mut self, mut game: Game, go: Go) {
//         if self.is_running() {
//             panic!(concat!(
//                 "Do not send the go command to the Search ",
//                 "until the ongoing search is finished.",
//             ));
//         }

//         let moves = go
//             .searchmoves
//             .map(|moves| {
//                 let mut vec = Vec::new();
//                 for lan_move in moves {
//                     let Some(chess_move) = game.lan_move(lan_move) else {
//                         continue;
//                     };
//                     vec.push(chess_move);
//                 }
//                 vec
//             })
//             .filter(|vec| !vec.is_empty())
//             .unwrap_or({
//                 let mut vec = Vec::new();
//                 game.explore()
//                     .for_each_legal_child_node(MoveOrdering::MvvLva, |_, chess_move| {
//                         vec.push(chess_move);
//                     });
//                 vec
//             });
//         let moves = {
//             let mut map = HashMap::new();
//             for chess_move in moves {
//                 map.insert(chess_move, None);
//             }
//             map
//         };
//         let deadline = go
//             .movetime
//             .or_else(|| {
//                 let turn = game.turn();
//                 let inc = match turn {
//                     Color::White => go.winc,
//                     Color::Black => go.binc,
//                 }
//                 .unwrap_or(Duration::ZERO);
//                 match turn {
//                     Color::White => go.wtime,
//                     Color::Black => go.btime,
//                 }
//                 .map(|time| time + inc)
//             })
//             .map(|d| Instant::now() + d);
//         _ = go.movestogo;
//         let nodes_max = go.nodes;
//         let depth_max = go.depth.map(|d| d - 1);
//         let mate = go.mate;
//         let ponder = go.ponder;
//         let infinite = go.infinite;
//         let pending_result = game.explore().check_ending().right().map(|_| SearchResult {
//             best_move: None,
//             ponder: None,
//         });
//         self.progress = Some(SearchProgress {
//             game,
//             moves,
//             deadline,
//             nodes_max,
//             depth_max,
//             mate,
//             is_pondering: ponder,
//             infinite,
//             pending_count: 0,
//             running_depth: 0,
//             pending_result,
//         });
//         if pending_result.is_some() {
//             return;
//         }

//         self.prepare();
//     }
//     pub fn stop(&mut self) -> SearchResult {
//         if let Some(pending) = self.get_progress().pending_result {
//             self.progress = None;
//             return pending;
//         }

//         self.server_send.send(ServerCommand::Cancel).unwrap();
//         self.recv();
//         let (best_move, _, _) = self.collect();
//         self.progress = None;
//         SearchResult {
//             best_move: Some(best_move.lan()),
//             ponder: None,
//         }
//     }
//     pub fn check(&mut self) -> Option<SearchResult> {
//         if !self.try_recv() {
//             return None;
//         }

//         let (best_move, should_stop, should_hold) = self.collect();
//         let progress = self.get_progress();

//         if should_stop {
//             let result = Some(SearchResult {
//                 best_move: Some(best_move.lan()),
//                 ponder: None,
//             });
//             if should_hold {
//                 progress.pending_result = result;
//             } else {
//                 self.progress = None;
//                 return result;
//             }
//         } else {
//             progress.running_depth += 1;
//             self.prepare();
//         }

//         None
//     }
//     pub fn clear_tt(&mut self) {
//         if self.is_running() {
//             panic!(concat!(
//                 "Do not try to clear the transposition table ",
//                 "until the ongoing search is finished.",
//             ));
//         }

//         self.server_send.send(ServerCommand::ClearHash).unwrap();
//     }
//     pub fn ponderhit(&mut self) {
//         let Some(ref mut progress) = self.progress else {
//             panic!(concat!(
//                 "Do not use the ponderhit method ",
//                 "unless there is an ongoing search.",
//             ));
//         };

//         progress.is_pondering = false;
//     }
//     fn collect(&mut self) -> (ChessMove, bool, bool) {
//         let progress = self.get_progress();
//         let mut best_move = None;
//         let mut score = None;
//         let mut nodes = 0;
//         let mut unfinished = false;

//         for (&chess_move, &result) in progress.moves.iter() {
//             let result = result.unwrap();
//             if score.is_none_or(|s| result.score > s) {
//                 score = Some(result.score);
//                 best_move = Some(chess_move);
//             }
//             nodes += result.nodes;
//             unfinished |= result.is_canceled;
//         }

//         let best_move = best_move.unwrap();
//         let score = score.unwrap();

//         let depth_fail = progress
//             .depth_max
//             .is_some_and(|max| progress.running_depth >= max);
//         let nodes_fail = progress.nodes_max.is_some_and(|max| nodes >= max);
//         let mate_fail = progress
//             .mate
//             .is_some_and(|n| score >= Score::Mating(n) || score <= Score::Mated(n));
//         let should_stop = unfinished | depth_fail | nodes_fail | mate_fail;
//         let only_depth = depth_fail & !nodes_fail & !mate_fail;
//         let should_hold = should_stop & (progress.is_pondering | (only_depth & progress.infinite));

//         (best_move, should_stop, should_hold)
//     }
//     fn prepare(&mut self) {
//         assert!(self.get_progress().pending_count == 0);
//         let progress = self.progress.as_mut().unwrap();
//         progress.pending_count = progress.moves.len();
//         let mut vec = Vec::with_capacity(progress.pending_count);
//         for &chess_move in progress.moves.keys() {
//             let mut game = progress.game.clone();
//             game.make_move(chess_move);
//             vec.push(SearchRequest {
//                 game,
//                 depth: progress.running_depth,
//                 nodes: progress.nodes_max,
//                 deadline: progress.deadline,
//             });
//         }
//         self.server_send
//             .send(ServerCommand::ProcessBatch(vec))
//             .unwrap();
//     }
//     fn recv(&mut self) {
//         while self.get_progress().pending_count > 0 {
//             let response = match self.server_recv.recv() {
//                 Ok(response) => response,
//                 Err(RecvError) => panic!("Search server disconnected!"),
//             };
//             self.update(response);
//             self.get_progress().pending_count -= 1;
//         }
//     }
//     fn try_recv(&mut self) -> bool {
//         while self.get_progress().pending_count > 0 {
//             let response = match self.server_recv.try_recv() {
//                 Ok(response) => response,
//                 Err(TryRecvError::Empty) => break,
//                 Err(TryRecvError::Disconnected) => panic!("Search server disconnected!"),
//             };
//             self.update(response);
//             self.get_progress().pending_count -= 1;
//         }

//         self.get_progress().pending_count == 0
//     }
//     fn update(&mut self, response: ServerResponse) {
//         let key = *self
//             .get_progress()
//             .moves
//             .keys()
//             .nth(response.batch_index)
//             .unwrap();
//         let maybe_result = self.get_progress().moves.get_mut(&key).unwrap();
//         let result = maybe_result.get_or_insert(response.result);
//         if !response.result.is_canceled {
//             *result = response.result;
//         }
//     }
//     fn get_progress(&mut self) -> &mut SearchProgress {
//         self.progress.as_mut().unwrap()
//     }
// }
