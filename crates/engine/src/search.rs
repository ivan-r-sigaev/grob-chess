
use crossbeam::channel::Select;
use game::{ParallelSearch, Transposition};
use position::LanMove;

use crate::uci::Go;

#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub struct SearchResult {
    best_move: Option<LanMove>,
}

#[allow(unused)]
#[derive(Debug)]
pub struct Search {
    search: ParallelSearch,
}

impl Search {
    #[allow(unused)]
    pub fn new() -> Self {
        const THREAD_COUNT: usize = 1;
        const TT_CAPACITY_IN_BYTES: usize = 16 * 1024 * 1024;
        const TT_CAPACITY: usize = TT_CAPACITY_IN_BYTES / size_of::<Transposition>();
        let search = ParallelSearch::new(THREAD_COUNT, TT_CAPACITY);
        Self { search }
    }
    #[allow(unused)]
    pub fn is_running(&self) -> bool {
        todo!()
    }
    #[allow(unused)]
    pub fn wait<'a>(&'a self, sel: &mut Select<'a>) -> usize {
        self.search.wait(sel)
    }
    #[allow(unused)]
    pub fn go(&mut self, go: Go) {
        todo!()
    }
    #[allow(unused)]
    pub fn stop(&mut self) -> SearchResult {
        todo!()
    }
    #[allow(unused)]
    pub fn check(&mut self) -> Option<SearchResult> {
        todo!()
    }
    #[allow(unused)]
    pub fn clear_tt(&mut self) {
        if self.is_running() {
            // TODO: should I panic?
            todo!()
        }

        self.search.clear_tt();
    }
}
