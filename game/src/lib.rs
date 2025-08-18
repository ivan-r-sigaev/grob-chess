/*
    TO BENCHMARK:
    - bitboard representation (classical / denser)
    - inlining (???)
*/
pub mod game;
pub mod move_list;
pub mod search;
pub mod transposition_table;

#[cfg(test)]
pub mod perft;
