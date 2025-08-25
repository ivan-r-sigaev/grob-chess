use crossbeam::channel::{Receiver, Select, TryRecvError};
use game::Game;
use std::{
    collections::VecDeque,
    fmt::Write,
    io::{self, stdin},
};

use crate::{
    search::{Search, SearchResult},
    uci::{Command, spawn_uci_parser},
};

const ENGINE_NAME: &str = "Pico Chess";
const AUTHOR_NAME: &str = "Ivan Sigaev";

#[derive(Debug)]
pub struct Server {
    should_quit: bool,
    command_recv: Receiver<io::Result<Command>>,
    pending_commands: VecDeque<Command>,
    game: Game,
    search: Search,
}

impl Server {
    pub fn new() -> Self {
        let should_quit = false;
        let command_recv = spawn_uci_parser(Box::new(stdin()));
        let search = Search::new();
        let game = Game::initial_position();
        Self {
            should_quit,
            command_recv,
            pending_commands: VecDeque::new(),
            game,
            search,
        }
    }
    pub fn run(&mut self) {
        while !self.should_quit {
            let mut sel = Select::new();
            if self.search.is_running() {
                let uci_index = sel.recv(&self.command_recv);
                let search_index = self.search.add_to_select(&mut sel);
                match sel.ready() {
                    index if index == uci_index => self.handle_commands(),
                    index if index == search_index => self.update_search(),
                    _ => unreachable!(),
                }
            } else {
                sel.recv(&self.command_recv);
                sel.ready();
                self.handle_commands();
            }
        }
    }
    fn handle_commands(&mut self) {
        while let Some(command) = self.try_recv_command() {
            if !self.pending_commands.is_empty() {
                self.pending_commands.push_back(command);
                break;
            }
            if !self.handle_command(command) {
                break;
            }
            if self.should_quit {
                break;
            }
        }
    }
    fn handle_command(&mut self, command: Command) -> bool {
        if self.search.is_running()
            && matches!(
                command,
                Command::UciNewGame | Command::Position(_) | Command::Go(_)
            )
        {
            self.pending_commands.push_back(command);
            return false;
        }

        match command {
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
            Command::Go(go) => self.search.go(self.game.clone(), go),
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

        true
    }
    fn update_search(&mut self) {
        let Some(result) = self.search.check() else {
            return;
        };
        Self::display_search_result(result);

        while let Some(command) = self.pending_commands.pop_front() {
            if !self.handle_command(command) {
                break;
            }
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
    fn try_recv_command(&self) -> Option<Command> {
        match self.command_recv.try_recv() {
            Ok(res) => Some(res.unwrap()),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => panic!("UCI reader has disconnected!"),
        }
    }
}
