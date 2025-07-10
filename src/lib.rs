/*
    TO BENCHMARK:
    - bitboard representation (classical / denser)
    - inlining (???)
*/
pub mod bitboard;
pub mod board;
pub mod game;
pub mod search;
pub mod transposition_table;

#[cfg(test)]
mod testing;
