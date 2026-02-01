use std::{collections::HashSet, sync::{Arc, Barrier, Mutex, atomic::{AtomicI32, AtomicU8, AtomicU64, AtomicUsize, Ordering}}, thread, time::Instant};

use crossbeam::{channel::{Receiver, Sender, bounded, unbounded}, deque::{Stealer, Worker}, utils::CachePadded};
use either::Either;
use grob_core::{
    game::{Game, movegen::ChessMove, walker::{GameEnding, GameTreeWalker, MoveOrdering}},
    pieces::Piece,
};
use parking_lot::RwLock;

use crate::{
    MAX_DEPTH, pv::PvTable, score::Score, transposition::{Transposition, TranspositionTable}, worker::sync::{ThreadSync, ThreadSyncMaster}
};

mod sync;

#[derive(Debug, Default, Clone)]
pub struct SearchRequest {
    pub game: Game,
    pub moveset: HashSet<ChessMove>,
    pub node_limit: Option<u64>,
    pub time_limit: Option<Instant>,
    pub depth_limit: u8,
}

/// The result of running a search on a chess game with the [`SearchServer`].
#[derive(Debug)]
pub struct SearchResult {
    // ...
}

/// Controls serveral worker threads to search chess positions.
#[derive(Debug)]
pub struct SearchServer {
    thread_group: Option<ThreadGroup>,
    tt: Arc<TranspositionTable>,
    worker_count: usize,
    search_request_send: Sender<SearchRequest>,
    search_request_recv: Receiver<SearchRequest>,
    search_result_send: Sender<SearchResult>,
    search_result_recv: Receiver<SearchResult>,
}

impl SearchServer {
    /// Creates a new search server with the specified number of worker threads
    /// and a hash table (transposition table) that is not larger than the 
    /// specified number of mebibytes.
    pub fn new(worker_count: usize, hash_table_mebibytes: usize) -> Self {
        let tt_capacity = hash_table_mebibytes * 1024 * 1024 / TranspositionTable::ITEM_SIZE;
        let tt = Arc::new(TranspositionTable::new(tt_capacity));
        let (search_request_send, search_request_recv) = bounded(1);
        let (search_result_send, search_result_recv) = unbounded();
        let mut server = Self { thread_group: None, tt, worker_count, search_request_send, search_request_recv, search_result_send, search_result_recv };
        server.create_thread_group(worker_count);
        server
    }
    /// Returns `true` if the server is currently running a search.
    pub fn is_searching(&self) -> bool {
        self.thread_group.as_ref().unwrap().sync_master.is_running()
    }
    /// Starts the search with the specified search request.
    /// 
    /// If there is an ongoing search this will stop it early.
    #[must_use]
    pub fn run_search(&self, search_request: SearchRequest) {
        if self.is_searching() {
            self.stop_search();
        }
        let node_limit = search_request.node_limit;
        let time_limit = search_request.time_limit;

        self.search_request_send.send(search_request);
        let thread_group = self.thread_group.as_ref().unwrap();
        let mut global_data = thread_group.global_data.write();
        global_data.node_counter.store(0, Ordering::Relaxed);
        global_data.node_limit = node_limit;
        global_data.time_limit = time_limit;

        thread_group.sync_master.go();
    }
    /// If there is an ongoing search this will stop it early.
    pub fn stop_search(&self) {
        if !self.is_searching() {
            return;
        }
        self.thread_group.as_ref().unwrap().sync_master.stop();
    }
    /// Returns the reference to the channel that is used to collect the search results.
    pub fn search_result_channel(&self) -> &Receiver<SearchResult> {
        &self.search_result_recv
    }
    /// Changers the number of worker threads.
    /// 
    /// If there is an ongoing search this will stop it early.
    pub fn set_worker_count(&mut self, worker_count: usize) {
        if self.is_searching() {
            self.stop_search();
        }
        if worker_count != self.worker_count {
            self.drop_thread_group();
            self.create_thread_group(worker_count);
        }
    }
    /// Changes the size of the hash table (transposition table) to be not larger
    /// than the specified number of mebibytes.
    /// 
    /// # Performance
    /// This operation is slow and should not be used during the search.
    pub fn resize_hash_table(&self, hash_table_mebibytes: usize) {
        let new_capacity = hash_table_mebibytes * 1024 * 1024 / TranspositionTable::ITEM_SIZE;
        self.tt.resize(new_capacity);
    }
    /// Removes all entries from the hash table (transposition table).
    /// 
    /// # Performance
    /// This operation is slow and should not be used during the search.
    pub fn clear_hash_table(&self) {
        self.tt.clear();
    }
    /// Constructs a new thread group and assigns it to [`SearchServer::thread_group`].
    /// 
    /// # Panics
    /// Panics if [`SearchServer::thread_group`] is not `None`.
    fn create_thread_group(&mut self, worker_count: usize) {
        assert!(self.thread_group.is_none(), "Must drop the current thread group before creating a new one!");
        let (sync_master, syncs) = ThreadSyncMaster::new(worker_count);
        let global_data = Arc::new(RwLock::new(GlobalData {
            stealers: Vec::new(),
            node_counter: Arc::new(CachePadded::new(AtomicU64::new(0))),
            node_limit: None,
            time_limit: None,
            tt: self.tt.clone(),
            prev_moveset: HashSet::new(),
            prev_game: None,
        }));

        for (thread_index, sync) in syncs.into_iter().enumerate() {
            let search_request_recv = self.search_request_recv.clone();
            let search_result_send = self.search_result_send.clone();
            let global_data = global_data.clone();
            let worker = Worker::new_fifo();
            global_data.write().stealers.push(worker.stealer());
            _ = thread::spawn(move || {
                // let task_data = Arc::new(RwLock::new(TaskData {
                //     completed_countdown: Arc::new(CachePadded::new(AtomicUsize::new(0))),
                //     alpha: Arc::new(CachePadded::new(AtomicI32::new(0))),
                //     beta: Arc::new(CachePadded::new(AtomicI32::new(0))),
                //     pv_table: Arc::new(Mutex::new(PVTable::new(MAX_DEPTH as usize))),
                //     game: Default::default(),
                //     ply: Default::default(),
                //     should_change_pv: false,
                // }));
                let task_data = Vec::new();
                let task_data_used_count = 0;
                let thread_data = ThreadData {
                    search_request_recv,
                    search_result_send,
                    sync,
                    global_data,
                    task_data,
                    task_data_used_count,
                    worker,
                    thread_index,
                };
                thread_data.run_worker_thread()
            });
        }

        self.worker_count = worker_count;
        self.thread_group = Some(ThreadGroup { sync_master, global_data });
    }
    /// Terminate the worker threads.
    fn drop_thread_group(&mut self) {
        self.thread_group.take().unwrap().sync_master.quit();
    }
}

impl Drop for SearchServer {
    fn drop(&mut self) {
        self.drop_thread_group();
    }
}

/// Data relevant to a group of worker threads.
#[derive(Debug)]
struct ThreadGroup {
    sync_master: ThreadSyncMaster,
    global_data: Arc<RwLock<GlobalData>>,
}

/// Data shared between the worker threads.
#[derive(Debug)]
struct GlobalData {
    stealers: Vec<Stealer<SubTask>>,
    node_counter: Arc<CachePadded<AtomicU64>>,
    node_limit: Option<u64>,
    time_limit: Option<Instant>,
    tt: Arc<TranspositionTable>,
    prev_moveset: HashSet<ChessMove>,
    prev_game: Option<Game>,
}

/// Data unique per each worker thread.
#[derive(Debug)]
struct ThreadData {
    search_request_recv: Receiver<SearchRequest>,
    search_result_send: Sender<SearchResult>,
    sync: ThreadSync,
    global_data: Arc<RwLock<GlobalData>>,
    task_data: Vec<Arc<RwLock<TaskData>>>,
    task_data_used_count: usize,
    worker: Worker<SubTask>,
    thread_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum NodeKind {
    Pv,
    Cut,
    All,
}

// ID: D1(PV0), D2(PV1), D3(PV2), ...

impl ThreadData {
    fn run_worker_thread(self) {
        loop {
            let wakeup_result = self.sync.go();
            if wakeup_result.should_quit {
                break;
            }

            let global_data = self.global_data.read_recursive();
            if wakeup_result.is_leader {
                let request = self.search_request_recv.recv().expect("Search server has disconnected!");
                todo!();
                // let task_data = self.task_data.write(); // TODO: allocate task data...
                let pv_table = task_data.pv_table.clone();
                // Try to reuse the PV from the previous game.
                if let Some(mut prev_game) = global_data.prev_game.clone() {
                    let mut perv_moveset = global_data.prev_moveset.clone();
                    while let Some(pv_move) = pv_table.lock().unwrap().pv().next() {
                        if prev_game.is_same_position(&request.game) && perv_moveset == request.moveset {
                            break;
                        }
                        pv_table.lock().unwrap().apply_pv_move();
                        prev_game.make_move(pv_move);
                        perv_moveset.clear();
                        _ = prev_game.walk().for_each_legal_move(
                            Default::default(),
                            None,
                            |_, chess_move| {
                                perv_moveset.insert(chess_move);
                            }
                        );
                    }
                }
                
                let mut best_move = pv_table.lock().unwrap().pv().nth(0);
                let mut ponder_move = pv_table.lock().unwrap().pv().nth(1);

                let pv_depth = pv_table.lock().unwrap().pv_len();
                let pv_depth_goal = request.depth_limit as usize;
                for depth in pv_depth..=pv_depth_goal {
                    // TODO:  
                    // TODO: for orginal position generate shared data...
                    // TODO: search the elder brother (if searched).    
                    // TODO: for each searched move generated tasks and push...
                    // TODO: apply
                }
                todo!()
            }

            todo!()
            // while let Ok(job) = self.job_recv.try_recv() {
            //     let mut game = job.data.game;
            //     let worst_score = Score::from_ending(GameEnding::Checkmate);
            //     let result = self.search(
            //         &mut game.walk(),
            //         job.data.depth,
            //         SearchConstraints {
            //             nodes_max: job.data.nodes,
            //             deadline: job.data.deadline,
            //         },
            //         worst_score,
            //         worst_score.prev(),
            //     );
            //     let result = SearchJobResult {
            //         result,
            //         id: job.id,
            //     };
            //     self.res_send.send(result).unwrap();
            // }

            // self.signal.stop();
        }
    }
    // fn search(&mut self, node: GameTreeWalker, ply: usize, kind: NodeKind, moves: Option<&Vec<ChessMove>>) {
    //     todo!()
    // }
}

/// Data unique per each worker thread that is also
/// shared with task stealers.
#[derive(Debug)]
struct TaskData {
    game: Game,
    ply: usize,
    should_change_pv: bool,
    pv_table: Arc<Mutex<PvTable>>,
    alpha: Arc<CachePadded<AtomicI32>>,
    beta: Arc<CachePadded<AtomicI32>>,
    completed_countdown: Arc<CachePadded<AtomicUsize>>,
}

#[derive(Debug)]
struct SubTask {
    chess_move: ChessMove,
    task_data: Arc<RwLock<TaskData>>,
}

// #[derive(Debug, Clone, Copy)]
// struct SearchConstraints {
//     pub nodes_max: Option<u64>,
//     pub deadline: Option<Instant>,
// }

// impl SearchConstraints {
//     pub fn nodes_fail(self, nodes: u64) -> bool {
//         self.nodes_max.is_some_and(|max| nodes > max)
//     }
//     pub fn time_fails(self) -> bool {
//         self.deadline.is_some_and(|d| Instant::now() > d)
//     }
// }

// #[derive(Debug)]
// struct Worker {
//     signal: SignalReceiver,
//     job_recv: Receiver<SearchJob>,
//     res_send: Sender<SearchJobResult>,
//     tt: Arc<TranspositionTable>,
// }

// impl Worker {
//     fn run(&mut self) {
//         loop {
//             if !self.signal.go() {
//                 break;
//             }

//             while let Ok(job) = self.job_recv.try_recv() {
//                 let mut game = job.data.game;
//                 let worst_score = Score::from_ending(GameEnding::Checkmate);
//                 let result = self.search(
//                     &mut game.walk(),
//                     job.data.depth,
//                     SearchConstraints {
//                         nodes_max: job.data.nodes,
//                         deadline: job.data.deadline,
//                     },
//                     worst_score,
//                     worst_score.prev(),
//                 );
//                 let result = SearchJobResult {
//                     result,
//                     id: job.id,
//                 };
//                 self.res_send.send(result).unwrap();
//             }

//             self.signal.stop();
//         }
//     }
//     fn search(
//         &mut self,
//         node: &mut GameTreeWalker,
//         depth: u8,
//         constraints: SearchConstraints,
//         mut alpha: Score,
//         beta: Score,
//     ) -> SearchResult {
//         if self.signal.should_stop() || constraints.time_fails() {
//             return self.evaluate(node, true);
//         }

//         let position = node.game();
//         let hash = position.zobrist();
//         if let Some(t) = self.tt.get(hash) {
//             'probe_hash: {
//                 if position.is_move_pseudo_legal(t.best_move) {
//                     break 'probe_hash;
//                 }
//                 if t.depth < depth {
//                     // TODO: should optimize for hash move here.
//                     break 'probe_hash;
//                 }
//                 return SearchResult {
//                     best_move: Some(t.best_move),
//                     score: t.score,
//                     nodes: 1,
//                     is_canceled: false,
//                 };
//             }
//         }

//         if depth == 0 {
//             return self.quiescence(node, alpha, beta);
//         }

//         let mut best_move = None;
//         let mut best_score = None;
//         let mut nodes = 1;
//         let mut is_canceled = false;
//         let maybe_ending = node.for_each_legal_move(MoveOrdering::MvvLva, |node, chess_move| {
//             let result = self.search(node, depth - 1, constraints, alpha, beta);
//             nodes += result.nodes;
//             if best_score.is_none_or(|score| result.score.prev() > score) {
//                 best_score = Some(result.score);
//                 best_move = Some(chess_move);
//             }

//             if result.score > alpha {
//                 alpha = result.score;
//             }

//             if result.score >= beta {
//                 node.exhaust_moves();
//                 return;
//             }

//             if constraints.nodes_fail(nodes)
//                 || constraints.time_fails()
//                 || self.signal.should_stop()
//             {
//                 is_canceled = true;
//                 node.exhaust_moves();
//             }
//         });
//         let score = match maybe_ending {
//             Some(ending) => Score::from_ending(ending),
//             None => {
//                 let best_move = best_move.unwrap();
//                 let score = best_score.unwrap();
//                 self.tt.insert(
//                     hash,
//                     Transposition {
//                         best_move,
//                         depth,
//                         score,
//                     },
//                 );
//                 score
//             }
//         };

//         SearchResult {
//             best_move,
//             score,
//             nodes,
//             is_canceled,
//         }
//     }
//     fn quiescence(&mut self, node: &mut GameTreeWalker, alpha: Score, beta: Score) -> SearchResult {
//         if self.signal.should_stop() {
//             return self.evaluate(node, true);
//         }

//         _ = (alpha, beta);

//         // TODO: implement quiescence search.
//         self.evaluate(node, false)
//     }
//     fn evaluate(&mut self, node: &mut GameTreeWalker, is_canceled: bool) -> SearchResult {
//         let nodes = 1;
//         let any_move = match node.check_ending() {
//             Either::Left(chess_move) => chess_move,
//             Either::Right(ending) => {
//                 return SearchResult {
//                     best_move: None,
//                     score: Score::from_ending(ending),
//                     nodes,
//                     is_canceled,
//                 };
//             }
//         };

//         let position = node.game();
//         let board = position.board();
//         let player = board.get_color(position.turn());
//         let queens = board.get_piece(Piece::Queen);
//         let rooks = board.get_piece(Piece::Rook);
//         let bishops = board.get_piece(Piece::Bishop);
//         let knights = board.get_piece(Piece::Knight);
//         let pawns = board.get_piece(Piece::Pawn);
//         let q_score = (queens & player).count() as i32 - (queens & !player).count() as i32;
//         let r_score = (rooks & player).count() as i32 - (rooks & !player).count() as i32;
//         let b_score = (bishops & player).count() as i32 - (bishops & !player).count() as i32;
//         let n_score = (knights & player).count() as i32 - (knights & !player).count() as i32;
//         let p_score = (pawns & player).count() as i32 - (pawns & !player).count() as i32;
//         let p = p_score + (n_score + b_score) * 3 + r_score * 5 + q_score * 9;
//         let score = Score::from_centipawns(p * 100);

//         SearchResult {
//             best_move: Some(any_move),
//             score,
//             nodes,
//             is_canceled,
//         }
//     }
// }
