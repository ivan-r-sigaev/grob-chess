/*
    TO BENCHMARK:
    - bitboard representation (classical / denser)
    - inlining (???)
*/
extern crate pico_chess_proc_macro;
pub mod game;
pub mod transposition_table;
pub mod search;

#[cfg(test)]
mod testing;
