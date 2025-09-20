use crossbeam::{
    channel::{Receiver, Sender},
    select,
};
use grob_core::Game;
use std::{
    collections::VecDeque,
    fmt::Write,
    io::{self, stdin},
};

use crate::{
    search::{SearchCommand, SearchResult, spawn_uci_server},
    uci::{Command, spawn_uci_parser},
};

const ENGINE_NAME: &str = "Grob";
const AUTHOR_NAME: &str = "Ivan Sigaev";

#[derive(Debug)]
pub struct Server {
    game: Game,
    command_recv: Receiver<io::Result<Command>>,
    pending_commands: VecDeque<Command>,
    search_send: Sender<SearchCommand>,
    search_recv: Receiver<SearchResult>,
    should_quit: bool,
    expecting_res: bool,
}

impl Server {
    pub fn new() -> Self {
        let command_recv = spawn_uci_parser(Box::new(stdin()));
        let (search_send, search_recv) = spawn_uci_server();
        let game = Game::initial_position();
        Self {
            game,
            command_recv,
            pending_commands: VecDeque::new(),
            search_send,
            search_recv,
            should_quit: false,
            expecting_res: false,
        }
    }
    pub fn run(&mut self) {
        while !self.should_quit {
            select! {
                recv(self.command_recv) -> result => {
                    _ = self.handle_command(result.unwrap().unwrap());
                }
                recv(self.search_recv) -> result => self.update_search(result.unwrap()),
            }
        }
    }
    fn handle_command(&mut self, command: Command) -> bool {
        if !self.pending_commands.is_empty()
            || (self.expecting_res
                && matches!(
                    command,
                    Command::UciNewGame | Command::Position(_) | Command::Go(_)
                ))
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
            Command::UciNewGame => self.search_send.send(SearchCommand::UciNewGame).unwrap(),
            Command::Position(game) => self.game = game,
            Command::Go(go) => {
                self.expecting_res = true;
                self.search_send
                    .send(SearchCommand::Go(Box::new(go), self.game.clone()))
                    .unwrap()
            }
            Command::Stop => {
                if self.expecting_res {
                    self.stop_search()
                }
            }
            Command::PonderHit => {
                if self.expecting_res {
                    self.search_send.send(SearchCommand::PonderHit).unwrap()
                }
            }
            Command::Quit => {
                if self.expecting_res {
                    self.stop_search();
                }
                self.should_quit = true;
            }
        }

        true
    }
    fn update_search(&mut self, res: SearchResult) {
        assert!(self.expecting_res);
        Self::display_search_result(res);
        self.expecting_res = false;

        while let Some(command) = self.pending_commands.pop_front() {
            if !self.handle_command(command) {
                break;
            }
        }
    }
    fn stop_search(&mut self) {
        self.search_send.send(SearchCommand::Stop).unwrap();
        let res = self.search_recv.recv().unwrap();
        self.update_search(res);
    }
    fn display_search_result(res: SearchResult) {
        let mut msg = String::from("bestmove ");
        match res.best_move {
            Some(best_move) => write!(msg, "{best_move}").unwrap(),
            None => write!(msg, "(none)").unwrap(),
        };
        if let Some(ponder) = res.ponder {
            write!(msg, " ponder {ponder}").unwrap();
        };
        println!("{msg}");
    }
}
