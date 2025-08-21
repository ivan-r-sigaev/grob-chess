use std::fmt::Write;
use crossbeam::channel::Select;
use game::Game;
use position::Position;

use crate::{search::{Search, SearchResult}, uci::{Command, UciChannel}};

#[derive(Debug)]
pub struct Server {
    should_quit: bool,
    uci: UciChannel,
    is_uci_blocked: bool,
    game: Game,
    search: Search,
}

const ENGINE_NAME: &str = "Pico Chess";
const AUTHOR_NAME: &str = "Ivan Sigaev";

impl Server {
    pub fn new() -> Self {
        let should_quit = false;
        let uci = UciChannel::spawn();
        let is_uci_blocked = false;
        let search = Search::new();
        let game = Game::from_position(Position::initial_position());
        Self { should_quit, uci, is_uci_blocked, game, search }
    }
    pub fn run(&mut self) {
        while !self.should_quit {
            let mut sel = Select::new();
            let uci_index = self.uci.wait(&mut sel);
            let search_index = self.search.wait(&mut sel);
            match sel.ready() {
                index if index == uci_index => self.handle_commands(),
                index if index == search_index => self.update_search(),
                _ => unreachable!(),
            }
        }
    }
    fn handle_commands(&mut self) {
        while self.uci.check().is_some() {
            if self.is_uci_blocked {
                break;
            }
            self.handle_command();
            if self.should_quit {
                break;
            }
        }
    }
    fn handle_command(&mut self) {
        if self.search.is_running() && matches!(
            self.uci.check().unwrap(),
            Command::UciNewGame |
            Command::Position(_) |
            Command::Go(_)
        ) {
            self.is_uci_blocked = true;
            return;
        }

        match self.uci.pop().unwrap() {
            Command::Uci => {
                println!("id name {ENGINE_NAME}");
                println!("id author {AUTHOR_NAME}");
                println!("uciok");
            }
            Command::IsReady => {
                println!("readyok");
            }
            Command::UciNewGame => self.search.clear_tt(),
            Command::Position(game) => self.game = game,
            Command::Go(go) => self.search.go(go),
            Command::Stop => {
                if self.search.is_running() {
                    self.stop_search()
                }
            }
            Command::PonderHit => {
                if self.search.is_running() {
                    self.search.ponderhit()
                }
            }
            Command::Quit => {
                if self.search.is_running() {
                    self.stop_search();
                }
                self.should_quit = true;
            }
        }
    }
    fn update_search(&mut self) {
        let Some(result) = self.search.check() else {
            return;
        };
        Self::display_search_result(result);
        
        if self.is_uci_blocked {
            self.is_uci_blocked = false;
            self.handle_commands();
        }
    }
    fn stop_search(&mut self) {
        let result = self.search.stop();
        Self::display_search_result(result);
    }
    fn display_search_result(result: SearchResult) {
        let mut msg = String::from("bestmove ");
        match result.best_move {
            Some(best_move) => write!(msg, "{best_move}").unwrap(),
            None => write!(msg, "(none)").unwrap(),
        };
        if let Some(ponder) = result.ponder {
            write!(msg, " ponder {ponder}").unwrap();
        };
        println!("{msg}");
    }
}
