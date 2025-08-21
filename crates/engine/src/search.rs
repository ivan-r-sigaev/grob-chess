use game::{ParallelSearch, Transposition, Waiter};
use position::LanMove;

use crate::uci::Go;

#[derive(Debug, Clone, Copy)]
pub struct SearchResult {
    pub best_move: Option<LanMove>,
    pub ponder: Option<LanMove>,
}

#[derive(Debug)]
pub struct Search {
    search: ParallelSearch,
}

impl Search {
    pub fn new() -> Self {
        const THREAD_COUNT: usize = 1;
        const TT_CAPACITY_IN_BYTES: usize = 16 * 1024 * 1024;
        const TT_CAPACITY: usize = TT_CAPACITY_IN_BYTES / size_of::<Transposition>();
        let search = ParallelSearch::new(THREAD_COUNT, TT_CAPACITY);
        Self { search }
    }
    pub fn is_running(&self) -> bool {
        todo!()
    }
    pub fn add_to_waiter<'a>(&'a self, _waiter: &mut Waiter<'a>) -> usize {
        todo!()
    }
    pub fn go(&mut self, _go: Go) {
        todo!()
    }
    pub fn stop(&mut self) -> SearchResult {
        todo!()
    }
    pub fn check(&mut self) -> Option<SearchResult> {
        todo!()
    }
    pub fn clear_tt(&mut self) {
        if self.is_running() {
            // TODO: should I panic?
            todo!()
        }

        self.search.clear_tt();
    }
    pub fn ponderhit(&mut self) {
        todo!()
    }
}
