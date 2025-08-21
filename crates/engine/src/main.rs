//! Engine
//!
//! This crate is the driver code for the engine.

use crate::uci::{Command, UciChannel};

mod uci;
mod search;

const ENGINE_NAME: &str = "Pico Chess";
const AUTHOR_NAME: &str = "Ivan Sigaev";

fn main() {
    let mut uci_channel = UciChannel::spawn();

    loop {
        let command = uci_channel.recv().unwrap();
        match command {
            Command::Uci => {
                println!("id name {ENGINE_NAME}");
                println!("id author {AUTHOR_NAME}");
                println!("uciok");
            }
            Command::IsReady => {
                println!("readyok");
            }
            Command::UciNewGame => todo!(),
            Command::Position(_game) => todo!(),
            Command::Go(_go) => todo!(),
            Command::Stop => todo!(),
            Command::PonderHit => todo!(),
            Command::Quit => break,
        }
    }
}
