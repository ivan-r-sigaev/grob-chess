/*
    TO BENCHMARK:
    - bitboard representation (classical / denser)
    - inlining (???)
*/
pub mod bitboard;
pub mod game;
pub mod search;
pub mod transposition_table;

#[cfg(test)]
mod testing;
