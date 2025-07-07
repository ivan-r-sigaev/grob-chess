/*
    TO BENCHMARK:
    - bitboard representation (classical / denser)
    - inlining (???)
*/
pub mod game;
pub mod transposition_table;
pub mod search;
pub mod table_generation;

#[cfg(test)]
mod testing;
