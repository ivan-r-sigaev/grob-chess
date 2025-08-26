use std::{sync::Arc, thread, time::Instant};

use crossbeam::{
    channel::{bounded, unbounded, Receiver, RecvError, SendError, Sender},
    select,
};

use crate::{
    search::{
        transposition::TranspositionTable,
        worker::{Job, WorkerGroup},
    },
    ChessMove, Game, Score,
};

/// A command for the parallel search server.
#[derive(Debug, Clone)]
pub enum ServerCommand {
    /// Start processing multiple [`SearchRequest`]s.
    ///
    /// Server will try to prioritize processing
    /// [`SearchRequest`]s in the same order as they
    /// apper in the batch, but it might return the
    /// [`ServerResponse`]s in a different order.
    /// Check [`ServerResponse::batch_index`] to find out about
    /// the position of the corresponding [`SearchRequest`]
    /// inside of the batch.
    ///
    /// Sending another batch while the previous one hasn't
    /// finished yet will have the same effect as sending
    /// [`ServerCommand::Cancel`] in between the batches.
    ProcessBatch(Vec<SearchRequest>),
    /// If the server is currenty processing a batch it will try
    /// to finish it ASAP, but the search quality will suffer.
    ///
    /// All [`SearchResult`]s finished in this way will have
    /// the [`SearchResult::is_canceled`] flag set to `true`.
    Cancel,
    /// Immediately clears all data from the transposition table.
    ///
    /// # Performance
    /// This is a slow operation and it may cause the ongoing search
    /// to miss its deadline.
    ClearHash,
    /// Immediately resize the transposition table to be as large
    /// as possible but no more than a specified number of mebibytes (MiB).
    ///
    /// Transposition table size limit cannot be smaller than 1 MiB.
    ///
    /// # Performance
    /// This is a slow operation and it may cause the ongoing search
    /// to miss its deadline.
    SetHashSize { max_mib: usize },
    /// Change the amount of worker threads to be used in the future
    /// searches.
    ///
    /// This will **NOT** influence the amount of threads used in the
    /// ongoing search.
    ///
    /// Search will always use at least one worker thread.
    SetWorkerCount(usize),
}

/// Request to search a position.
#[derive(Debug, Clone)]
pub struct SearchRequest {
    /// Game position to search.
    pub game: Game,
    /// Depth of the search.
    pub depth: u64,
    /// Searched nodes limit.
    pub nodes: Option<u64>,
    /// Search time limit.
    pub deadline: Option<Instant>,
}

/// Processing results for a [`SearchRequest`] originating from
/// [`ServerCommand::ProcessBatch`].
#[derive(Debug, Clone, Copy)]
pub struct ServerResponse {
    /// Index of the corresponding [`SearchRequest`] inside of
    /// [`ServerCommand::ProcessBatch`].
    pub batch_index: usize,
    /// Result of the search.
    pub result: SearchResult,
}

/// Result of searching a position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchResult {
    /// Proposed score of the position.
    pub score: Score,
    /// Number of nodes searched.
    pub nodes: u64,
    /// Proposed best move (or `None` if no legal moves are available).
    pub best_move: Option<ChessMove>,
    /// Whether the corresponding[`SearchRequest`] was abruptly
    /// canceled with [`ServerCommand::Cancel`].
    pub is_canceled: bool,
}

/// Spawns a thread that will process the [`ServerCommand`]s and return
/// appropriate [`ServerResponse`]s.
///
/// The server will exit gracefully whenever at least one of its channels
/// gets disconnected.
pub fn spawn_search_server(
    worker_count: usize,
    tt_max_capacity_mib: usize,
) -> (Sender<ServerCommand>, Receiver<ServerResponse>) {
    let (cmd_send, cmd_recv) = unbounded();
    let (rsp_send, rsp_recv) = bounded(0);
    thread::spawn(move || {
        SearchScheduler::new(worker_count, tt_max_capacity_mib, rsp_send, cmd_recv).run()
    });
    (cmd_send, rsp_recv)
}

/// Parallel search scheduler.
#[derive(Debug)]
struct SearchScheduler {
    workers: WorkerGroup,
    rsp_send: Sender<ServerResponse>,
    cmd_recv: Receiver<ServerCommand>,
    job_send: Sender<Job>,
    res_recv: Receiver<ServerResponse>,
    pending_count: usize,
    worker_count: usize,
    tt: Arc<TranspositionTable>,
}

/// Simple utility type to improve control flow.
#[derive(Debug)]
struct ShouldQuit;

type Result = std::result::Result<(), ShouldQuit>;

impl SearchScheduler {
    /// Constructs a new [`SearchScheduler`].
    ///
    /// Transposition table capacity (MiB) and worker count
    /// are clamped to the be `>= 1`.
    fn new(
        worker_count: usize,
        tt_max_capacity_mib: usize,
        rsp_send: Sender<ServerResponse>,
        cmd_recv: Receiver<ServerCommand>,
    ) -> Self {
        let (job_send, job_recv) = unbounded();
        let (res_send, res_recv) = unbounded();
        let tt_capacity = tt_max_capacity_mib.max(1) * 1024 * 1024 / TranspositionTable::ITEM_SIZE;
        let tt = Arc::new(TranspositionTable::new(tt_capacity));
        let workers = WorkerGroup::new(worker_count.max(1), job_recv, res_send, tt.clone());

        Self {
            workers,
            rsp_send,
            cmd_recv,
            job_send,
            res_recv,
            worker_count,
            pending_count: 0,
            tt,
        }
    }
    /// Run the scheduler's command execution loop.
    ///
    /// This function will exit gracefully once either
    /// [`SearchScheduler::rsp_send`] or [`SearchScheduler::cmd_recv`]
    /// disconnects.
    fn run(&mut self) {
        loop {
            if let Err(ShouldQuit) = self.run_inner() {
                break;
            }
        }
    }
    /// Run the body of the scheduler's loop.
    ///
    /// Returning `Err(ShouldQuit)` will cause the scheduler
    ///  to finish running.
    fn run_inner(&mut self) -> Result {
        select! {
            recv(self.cmd_recv) -> result => {
                let cmd = result.map_err(|RecvError| ShouldQuit)?;
                self.handle_command(cmd)
            }
            recv(self.res_recv) -> result => self.forward_response(result.unwrap()),
        }
    }
    /// Process the server command.
    fn handle_command(&mut self, cmd: ServerCommand) -> Result {
        match cmd {
            ServerCommand::ProcessBatch(batch) => self.process_batch(batch)?,
            ServerCommand::Cancel => self.cancel()?,
            ServerCommand::ClearHash => self.tt.clear(),
            ServerCommand::SetHashSize { max_mib } => self.set_hash_size(max_mib),
            ServerCommand::SetWorkerCount(worker_count) => self.worker_count = worker_count,
        }
        Ok(())
    }
    /// Forward the search result to the user.
    fn forward_response(&mut self, rsp: ServerResponse) -> Result {
        self.rsp_send.send(rsp).map_err(|SendError(_)| ShouldQuit)?;
        self.pending_count -= 1;
        Ok(())
    }
    /// Execute [`ServerCommand::ProcessBatch`].
    fn process_batch(&mut self, batch: Vec<SearchRequest>) -> Result {
        if self.workers.signaler().is_running() {
            self.cancel()?;
        }

        self.pending_count = batch.len();
        for (batch_index, request) in batch.into_iter().enumerate() {
            let job = Job {
                request,
                batch_index,
            };
            self.job_send.send(job).unwrap();
        }
        self.workers.signaler().go();
        Ok(())
    }
    /// Execute [`ServerCommand::Cancel`].
    fn cancel(&mut self) -> Result {
        self.workers.signaler().stop();
        while self.pending_count != 0 {
            let rsp = self.res_recv.recv().map_err(|RecvError| ShouldQuit)?;
            self.forward_response(rsp)?;
        }
        self.workers.resize(self.worker_count);
        Ok(())
    }
    /// Execute [`ServerCommand::SetHashSize`].
    fn set_hash_size(&mut self, max_mib: usize) {
        let new_capacity = max_mib * 1024 * 1024 / TranspositionTable::ITEM_SIZE;
        self.tt.resize(new_capacity);
    }
}
