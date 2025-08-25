use std::{sync::Arc, time::Instant};

use crossbeam::channel::{unbounded, Receiver, Select, Sender};

use crate::{
    search::{
        transposition::TranspositionTable,
        worker::{Job, JobResult, WorkerGroup},
    },
    ChessMove, Game, Score,
};

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
    workers: WorkerGroup,
    job_send: Sender<Job>,
    res_recv: Receiver<JobResult>,
    results: Vec<JobResult>,
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
    pub fn new(thread_count: usize, tt_capacity_mb: usize) -> Self {
        assert!(thread_count != 0, "Thread count must be at least one!");

        let tt_capacity = tt_capacity_mb / TranspositionTable::ITEM_SIZE;
        let (job_send, job_recv) = unbounded();
        let (res_send, res_recv) = unbounded();
        let tt = Arc::new(TranspositionTable::new(tt_capacity));
        let workers = WorkerGroup::new(thread_count, job_recv, res_send, tt.clone());

        Self {
            workers,
            job_send,
            res_recv,
            results: Vec::new(),
            jobs_count: 0,
            tt,
        }
    }
    /// Returns `true` if the parallel search currently running.
    pub fn is_searching(&self) -> bool {
        self.workers.signaler().is_running()
    }
    /// Returns how many search jobs are running/prepared to run.
    pub fn jobs_count(&self) -> usize {
        self.jobs_count
    }
    /// Returns how many search jobs are not yet completed.
    pub fn pending_count(&self) -> usize {
        self.jobs_count() - self.results.len()
    }
    /// Makes a selector wait until some search jobs are completed.
    ///
    /// # Panics
    /// Panics if search is not currently running.
    pub fn add_to_select<'a>(&'a self, sel: &mut Select<'a>) -> usize {
        if !self.is_searching() {
            panic!("Search is paused.");
        }

        sel.recv(&self.res_recv)
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

        if self.pending_count() == 0 {
            Some(self.collect_results())
        } else {
            None
        }
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
        self.workers.signaler().go();
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

        self.workers.signaler().stop();
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
        assert!(self.is_searching() && self.pending_count() == 0);
        self.jobs_count = 0;
        self.workers.signaler().stop();

        let mut result = Vec::with_capacity(self.results.len());
        self.results.sort_by(|a, b| a.index.cmp(&b.index));
        for job_result in self.results.drain(..) {
            result.push(job_result.res);
        }

        result
    }
}
