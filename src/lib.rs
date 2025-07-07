/*
    TO BENCHMARK:
    - bitboard representation (classical / denser)
    - inlining (???)
*/
pub mod game;
pub mod search;
pub mod table_generation;
pub mod transposition_table;

#[cfg(test)]
mod testing;
