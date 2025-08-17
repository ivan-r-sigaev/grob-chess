use crate::command::Command;
use std::io::{self, BufRead, BufReader};

mod command;

const ENGINE_NAME: &str = "Pico Chess";
const AUTHOR_NAME: &str = "Ivan Sigaev";

fn main() -> Result<(), io::Error> {
    let handle = io::stdin().lock();
    let mut lines = BufReader::new(handle).lines();
    for result in &mut lines {
        let Ok(command) = result?.parse::<Command>() else {
            continue;
        };
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
    Ok(())
}
