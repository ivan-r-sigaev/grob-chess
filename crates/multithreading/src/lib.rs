//! Multithreading
//!
//! This crate provides types related to multithreading.

pub use search::{ParallelSearch, Score, SearchResult};
pub use transposition::{Transposition, TranspositionTable};

mod cache;
mod search;
mod transposition;
