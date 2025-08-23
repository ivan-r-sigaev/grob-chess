use std::{
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc, Barrier,
    },
    thread,
    time::Instant,
};

use board::Piece;
use crossbeam::{
    channel::{unbounded, Receiver, Sender},
    utils::CachePadded,
};
use either::Either;
use position::ChessMove;

use crate::{Game, GameEnding, GameSearch, Transposition, TranspositionTable, Waiter};

/// How advantageous is a chess position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Score {
    /// Length of a forced mate sequence in turns (good).
    Mating(u64),
    /// Length of a forced mate sequence in turns (bad).
    Mated(u64),
    /// Score in centi-pawns (1/100 of a pawn).
    Cp(i32),
}

impl Score {
    /// Returns the socre for the [`GameEnding`].
    pub fn ending(ending: GameEnding) -> Self {
        match ending {
            GameEnding::Stalemate => Self::Cp(0),
            GameEnding::Checkmate => Self::Mated(0),
        }
    }
    /// Returns the score for the other player on the previous turn.
    pub fn prev(self) -> Self {
        match self {
            Score::Mating(n) => Score::Mated(n),
            Score::Mated(n) => Score::Mating(n + 1),
            Score::Cp(i) => Score::Cp(-i),
        }
    }
    /// Returns the score for the other player on the next turn.
    pub fn next(self) -> Self {
        match self {
            Score::Mating(n) => Score::Mated(n - 1),
            Score::Mated(n) => Score::Mating(n),
            Score::Cp(i) => Score::Cp(-i),
        }
    }
}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Score {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Score::Mating(n1), Score::Mating(n2)) => n1.cmp(n2).reverse(),
            (Score::Mating(_), Score::Mated(_)) => std::cmp::Ordering::Greater,
            (Score::Mating(_), Score::Cp(_)) => std::cmp::Ordering::Greater,
            (Score::Mated(_), Score::Mating(_)) => std::cmp::Ordering::Less,
            (Score::Mated(n1), Score::Mated(n2)) => n1.cmp(n2),
            (Score::Mated(_), Score::Cp(_)) => std::cmp::Ordering::Less,
            (Score::Cp(_), Score::Mating(_)) => std::cmp::Ordering::Less,
            (Score::Cp(_), Score::Mated(_)) => std::cmp::Ordering::Greater,
            (Score::Cp(i1), Score::Cp(i2)) => i1.cmp(i2),
        }
    }
}

/// Result of searching a position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchResult {
    /// Best move in a position
    ///
    /// `None` only if there are no legal moves.
    pub best_move: Option<ChessMove>,
    /// Position's quality for the player who is making a move.
    pub score: Score,
    /// How many non-unique positions in the game tree were searched.
    pub nodes: u64,
    /// `true` if search was unable to be fully completed for any reason.
    pub unfinished: bool,
}

/// Parallel search scheduler.
#[derive(Debug)]
pub struct ParallelSearch {
    #[allow(dead_code)]
    workers: Vec<Worker>,
    job_send: Sender<Job>,
    res_recv: Receiver<JobResult>,
    results: Vec<JobResult>,
    signal: Arc<CachePadded<AtomicU8>>,
    wakeup: Arc<Barrier>,
    jobs_count: usize,
    tt: Arc<TranspositionTable>,
}

impl ParallelSearch {
    /// Construct a new [`ParallelSearch`] with the specified amount of
    /// worker threads and a given transposition table capacity.
    ///
    /// # Panics
    /// - Panics if `thread_count` is zero.
    /// - Panics if `tt_capacity` is zero.
    /// - Panics if `tt_capacity * size_of::<Transposition>()` exceeds `isize::MAX`.
    pub fn new(thread_count: usize, tt_capacity: usize) -> Self {
        assert!(thread_count != 0, "Thread count must be at least one!");

        let mut workers = Vec::with_capacity(thread_count);
        let (job_send, job_recv) = unbounded();
        let (res_send, res_recv) = unbounded();
        let signal = Arc::new(CachePadded::new(AtomicU8::new(SIGNAL_STOP)));
        let wakeup = Arc::new(Barrier::new(thread_count + 1));
        let tt = Arc::new(TranspositionTable::new(tt_capacity));

        for _ in 0..thread_count {
            let worker = Worker::new(
                job_recv.clone(),
                res_send.clone(),
                signal.clone(),
                wakeup.clone(),
                tt.clone(),
            );
            workers.push(worker);
        }

        Self {
            workers,
            job_send,
            res_recv,
            results: Vec::new(),
            signal,
            jobs_count: 0,
            wakeup,
            tt,
        }
    }
    /// Returns `true` if the parallel search currently running.
    pub fn is_searching(&self) -> bool {
        self.signal.load(Ordering::Relaxed) == SIGNAL_GO
    }
    /// Returns how many search jobs are running/prepared to run.
    pub fn jobs_count(&self) -> usize {
        self.jobs_count
    }
    /// Returns how many search jobs are not yet completed.
    pub fn pending_count(&self) -> usize {
        self.jobs_count() - self.results.len()
    }
    /// Tells the waiter to wait until some search jobs are completed.
    ///
    /// # Panics
    /// Panics if search is not currently running.
    pub fn add_to_waiter<'a>(
        &'a self,
        waiter: &mut Waiter<'a>,
        deadline: Option<Instant>,
    ) -> usize {
        if !self.is_searching() {
            panic!("Search is paused.");
        }

        waiter.add(&self.res_recv, deadline)
    }
    /// Tries to collect the search results if all jobs are completed.
    ///
    /// # Panics
    /// Panics if search is not currently running.
    pub fn try_collect(&mut self) -> Option<Vec<SearchResult>> {
        if !self.is_searching() {
            panic!("Search is paused.");
        }

        while let Ok(search_result) = self.res_recv.try_recv() {
            self.results.push(search_result);
        }

        if self.pending_count() != 0 {
            return None;
        }

        self.signal.store(SIGNAL_STOP, Ordering::SeqCst);
        Some(self.collect_results())
    }
    /// Prepares a search job.
    ///
    /// # Panics
    /// Panics if search is already running.
    pub fn prepare_search(
        &mut self,
        game: Game,
        depth: u64,
        nodes_max: Option<u64>,
        deadline: Option<Instant>,
    ) -> usize {
        if self.is_searching() {
            panic!("Search is already running.");
        }

        let index = self.jobs_count;
        self.jobs_count += 1;
        let search_job = Job {
            game,
            depth,
            nodes_max,
            deadline,
            index,
        };
        self.job_send.send(search_job).unwrap();
        index
    }
    /// Starts to search the prepared search jobs.
    ///
    /// # Panics
    /// Panics if search is alraedy running.
    pub fn go(&mut self) {
        if self.is_searching() {
            panic!("Search is already running.");
        }

        self.results.reserve(self.jobs_count);
        self.signal.store(SIGNAL_GO, Ordering::SeqCst);
        self.wakeup.wait();
    }
    /// Forces all the jobs to immediately be completed even if
    /// it means that they will not be fully searched.
    ///
    /// # Panics
    /// Panics if search is not currently running.
    pub fn stop(&mut self) -> Vec<SearchResult> {
        if !self.is_searching() {
            panic!("Search is paused.");
        }

        self.signal.store(SIGNAL_STOP, Ordering::SeqCst);
        while self.pending_count() != 0 {
            self.results.push(self.res_recv.recv().unwrap());
        }

        self.collect_results()
    }
    /// Same as [`ParallelSearch::stop`] but also terminates all the workers.
    ///
    /// # Panics
    /// Panics if search is not currently running.
    pub fn quit(mut self) -> Vec<SearchResult> {
        if !self.is_searching() {
            panic!("Search is paused.");
        }

        self.signal.store(SIGNAL_QUIT, Ordering::SeqCst);
        while self.pending_count() != 0 {
            self.results.push(self.res_recv.recv().unwrap());
        }

        self.collect_results()
    }
    /// Clears the transposition table.
    pub fn clear_tt(&mut self) {
        self.tt.clear();
    }
    fn collect_results(&mut self) -> Vec<SearchResult> {
        assert!(self.pending_count() == 0);
        let mut result = Vec::with_capacity(self.results.len());
        self.results.sort_by(|a, b| a.index.cmp(&b.index));
        for job_result in self.results.drain(..) {
            result.push(job_result.res);
        }
        self.jobs_count = 0;

        result
    }
}

impl Drop for ParallelSearch {
    fn drop(&mut self) {
        self.signal.store(SIGNAL_QUIT, Ordering::SeqCst);
    }
}

#[derive(Debug, Clone)]
struct Job {
    game: Game,
    depth: u64,
    nodes_max: Option<u64>,
    deadline: Option<Instant>,
    index: usize,
}

#[derive(Debug, Clone, Copy)]
struct JobResult {
    pub res: SearchResult,
    pub index: usize,
}

const SIGNAL_STOP: u8 = 0;
const SIGNAL_GO: u8 = 1;
const SIGNAL_QUIT: u8 = 2;

#[derive(Debug)]
struct Worker(Option<thread::JoinHandle<()>>);

impl Drop for Worker {
    fn drop(&mut self) {
        let result = self.0.take().map(|h| h.join());

        // Ignore error if already panicking.
        if !thread::panicking() {
            result.unwrap().unwrap()
        }
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

impl Worker {
    fn new(
        job_recv: Receiver<Job>,
        res_send: Sender<JobResult>,
        signal: Arc<CachePadded<AtomicU8>>,
        wakeup: Arc<Barrier>,
        tt: Arc<TranspositionTable>,
    ) -> Self {
        let handle = thread::spawn(move || loop {
            wakeup.wait();

            while let Ok(job) = job_recv.try_recv() {
                let mut game = job.game;
                let worst_score = Score::ending(GameEnding::Checkmate);
                let res = search(
                    &mut game.search(),
                    tt.clone(),
                    job.depth,
                    SearchConstraints {
                        nodes_max: job.nodes_max,
                        deadline: job.deadline,
                    },
                    signal.clone(),
                    worst_score,
                    worst_score.prev(),
                );
                let result = JobResult {
                    res,
                    index: job.index,
                };
                res_send.send(result).unwrap();
            }

            if signal.load(Ordering::Relaxed) == SIGNAL_QUIT {
                break;
            }
        });
        Self(Some(handle))
    }
}

fn search(
    node: &mut GameSearch,
    tt: Arc<TranspositionTable>,
    depth: u64,
    constraints: SearchConstraints,
    signal: Arc<CachePadded<AtomicU8>>,
    mut alpha: Score,
    beta: Score,
) -> SearchResult {
    if signal.load(Ordering::Relaxed) != SIGNAL_GO || constraints.time_fails() {
        return evaluate(node, true);
    }

    let position = node.game().position();
    let hash = position.zobrist();
    if let Some(t) = tt.get_exact(hash) {
        'probe_hash: {
            if position.is_move_applicable(t.best_move) {
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
                unfinished: false,
            };
        }
    }

    if depth == 0 {
        return quiescence(node, signal, alpha, beta);
    }

    let mut best_move = None;
    let mut best_score = None;
    let mut nodes = 1;
    let mut unfinished = false;
    let maybe_ending = node.for_each_legal_child_node(|node, chess_move| {
        let result = search(
            node,
            tt.clone(),
            depth - 1,
            constraints,
            signal.clone(),
            alpha,
            beta,
        );
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
            || signal.load(Ordering::Relaxed) != SIGNAL_GO
        {
            unfinished = true;
            node.exhaust_moves();
        }
    });
    let score = match maybe_ending {
        Some(ending) => Score::ending(ending),
        None => {
            let best_move = best_move.unwrap();
            let score = best_score.unwrap();
            tt.insert(
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
        unfinished,
    }
}

fn quiescence(
    node: &mut GameSearch,
    signal: Arc<CachePadded<AtomicU8>>,
    alpha: Score,
    beta: Score,
) -> SearchResult {
    if signal.load(Ordering::Relaxed) != SIGNAL_GO {
        return evaluate(node, true);
    }

    _ = (alpha, beta);

    // TODO: implement quiescence search.
    evaluate(node, false)
}

fn evaluate(node: &mut GameSearch, unfinished: bool) -> SearchResult {
    let nodes = 1;
    let any_move = match node.check_ending() {
        Either::Left(chess_move) => chess_move,
        Either::Right(ending) => {
            return SearchResult {
                best_move: None,
                score: Score::ending(ending),
                nodes,
                unfinished,
            }
        }
    };

    let position = node.game().position();
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
        unfinished,
    }
}
