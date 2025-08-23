use crossbeam::channel::{Receiver, Select, Sender, TryRecvError, unbounded};
use position::{Game, LanMove};
use std::{
    collections::VecDeque,
    error, fmt,
    io::{self, BufRead, BufReader},
    result,
    str::FromStr,
    thread,
    time::Duration,
};

use crate::uci::cursor::Cursor;

mod cursor;

/// A convenience wrapper to be able to block on command line input.
#[derive(Debug)]
pub struct UciChannel {
    handle: Option<thread::JoinHandle<Result<()>>>,
    command_recv: Receiver<Command>,
    commands: VecDeque<Command>,
}

type Result<T> = result::Result<T, Error>;

#[allow(dead_code)]
#[derive(Debug)]
pub enum Error {
    Disconnected,
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl error::Error for Error {}

impl UciChannel {
    pub fn spawn() -> Self {
        let (command_send, command_recv) = unbounded();
        let commands = VecDeque::new();
        let handle = Some(thread::spawn(|| run_uci_channel(command_send)));
        Self {
            handle,
            command_recv,
            commands,
        }
    }
    pub fn add_to_select<'a>(&'a self, sel: &mut Select<'a>) -> usize {
        sel.recv(&self.command_recv)
    }
    pub fn check(&mut self) -> Option<&Command> {
        loop {
            let command = match self.command_recv.try_recv() {
                Ok(command) => command,
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => panic!("UciChannel got disconnected!"),
            };
            self.commands.push_back(command);
        }
        self.commands.back()
    }
    pub fn pop(&mut self) -> Option<Command> {
        self.check();
        self.commands.pop_back()
    }
    fn drop_inplace(&mut self) -> thread::Result<Result<()>> {
        self.handle.take().map(|h| h.join()).unwrap_or(Ok(Ok(())))
    }
}

impl Drop for UciChannel {
    fn drop(&mut self) {
        let result = self.drop_inplace();

        // Ignore error if already panicking.
        if thread::panicking() {
            return;
        }

        result.unwrap().unwrap()
    }
}

#[derive(Debug, Clone)]
pub enum Command {
    Uci,
    IsReady,
    UciNewGame,
    Position(Game),
    Go(Go),
    Stop,
    PonderHit,
    Quit,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Go {
    pub searchmoves: Option<Vec<LanMove>>,
    pub ponder: bool,
    pub wtime: Option<Duration>,
    pub btime: Option<Duration>,
    pub winc: Option<Duration>,
    pub binc: Option<Duration>,
    pub movestogo: Option<u64>,
    pub depth: Option<u64>,
    pub nodes: Option<u64>,
    pub mate: Option<u64>,
    pub movetime: Option<Duration>,
    pub infinite: bool,
}

impl FromStr for Command {
    type Err = ();

    fn from_str(s: &str) -> result::Result<Self, Self::Err> {
        let mut cursor = Cursor::new(s);
        let Some(token) = cursor.next_token() else {
            return Err(());
        };
        Ok(match token {
            "uci" => Command::Uci,
            "isready" => Command::IsReady,
            "ucinewgame" => Command::UciNewGame,
            "position" => cursor.parse_position(),
            "go" => cursor.parse_go(),
            "stop" => Command::Stop,
            "ponderhit" => Command::PonderHit,
            "quit" => Command::Quit,
            _ => return Err(()),
        })
    }
}

impl Cursor<'_> {
    fn parse_position(&mut self) -> Command {
        let has_fen = self.next_token() == Some("fen");
        let fen_str = self.until_token("moves");
        let maybe_fen = match has_fen {
            true => Some(fen_str.to_owned()),
            false => None,
        };

        let maybe_position = match maybe_fen {
            Some(fen) => Game::try_from_fen(&fen).ok(),
            None => Some(Game::initial_position()),
        };

        let mut maybe_game = maybe_position;
        if let Some(ref mut game) = maybe_game {
            while let Some(lan_move) = self.next_token().and_then(|t| t.parse::<LanMove>().ok()) {
                let Some(chess_move) = game.lan_move(lan_move) else {
                    break;
                };
                if !game.try_make_move(chess_move) {
                    break;
                }
            }
        }

        let game = maybe_game.unwrap_or_else(Game::initial_position);
        Command::Position(game)
    }
    fn parse_go(&mut self) -> Command {
        let mut go = Go::default();
        while let Some(token) = self.next_token() {
            match token {
                "searchmoves" => {
                    let mut moves = Vec::new();
                    let mut peek = self.clone();
                    while let Some(lan_move) =
                        peek.next_token().and_then(|t| t.parse::<LanMove>().ok())
                    {
                        *self = peek.clone();
                        moves.push(lan_move);
                    }
                    go.searchmoves = go.searchmoves.or(Some(moves));
                }
                "ponder" => go.ponder = true,
                "wtime" => go.wtime = go.wtime.or(self.try_parse_millis()),
                "btime" => go.btime = go.btime.or(self.try_parse_millis()),
                "winc" => go.winc = go.winc.or(self.try_parse_millis()),
                "binc" => go.binc = go.binc.or(self.try_parse_millis()),
                "movetime" => go.movetime = go.movetime.or(self.try_parse_millis()),
                "movestogo" => go.movestogo = go.movestogo.or(self.try_parse_u64()),
                "depth" => go.depth = go.depth.or(self.try_parse_u64()),
                "nodes" => go.nodes = go.nodes.or(self.try_parse_u64()),
                "mate" => go.mate = go.mate.or(self.try_parse_u64()),
                "infinite" => go.infinite = true,
                _ => (),
            }
        }
        Command::Go(go)
    }
    fn try_parse_millis(&mut self) -> Option<Duration> {
        self.try_parse_u64().map(Duration::from_millis)
    }
    fn try_parse_u64(&mut self) -> Option<u64> {
        let mut peek = self.clone();
        let res = peek.next_token().and_then(|t| t.parse::<u64>().ok());
        if res.is_some() {
            *self = peek;
        }
        res
    }
}

fn run_uci_channel(command_send: Sender<Command>) -> Result<()> {
    let handle = io::stdin().lock();
    let mut lines = BufReader::new(handle).lines();
    for result in &mut lines {
        let Ok(command) = result.map_err(Error::Io)?.parse::<Command>() else {
            continue;
        };
        command_send
            .send(command)
            .map_err(|_| Error::Disconnected)?;
    }
    Ok(())
}
