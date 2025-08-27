use crossbeam::channel::{Receiver, SendError, bounded};
use game::{Game, LanMove};
use std::{
    io::{self, BufRead, BufReader, Read},
    result,
    str::FromStr,
    thread,
    time::Duration,
};

use crate::uci_cursor::Cursor;

/// Spawns a thread that will parse UCI commands from a given `Read` trait object
/// and returns a channel from it.
///
/// Thread will exit gracefully if the reciever diconnects.
///
/// Thread will forward the error and exit gracefully if the read causes an error.
pub fn spawn_uci_parser(read: Box<dyn Read + Send>) -> Receiver<io::Result<Command>> {
    let (s, r) = bounded(0);
    _ = thread::spawn(move || {
        let mut lines = BufReader::new(read).lines();
        for result in &mut lines {
            match result {
                Ok(line) => {
                    let Ok(command) = line.parse::<Command>() else {
                        continue;
                    };
                    if let Err(SendError(_)) = s.send(Ok(command)) {
                        break;
                    }
                }
                Err(e) => {
                    _ = s.send(Err(e));
                    break;
                }
            };
        }
    });
    r
}

/// A UCI command.
#[derive(Debug, Clone)]
pub enum Command {
    /// \>\> uci - UCI handshake.
    Uci,
    /// \>\> isready - UCI ping.
    IsReady,
    /// \>\> ucinewgame - clear hash table.
    UciNewGame,
    /// \>\> position ... - setup position.
    Position(Game),
    /// \>\> go ... - start search.
    Go(Go),
    /// \>\> stop - stop search.
    Stop,
    /// \>\> ponderhit - exit pondering mode.
    PonderHit,
    /// \>\> quit - terminate the program.
    Quit,
}

/// Parameters for the "go" UCI command.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Go {
    /// Search only theese moves.
    pub searchmoves: Option<Vec<LanMove>>,
    /// Search in ponder mode.
    pub ponder: bool,
    /// Remaining time for white.
    pub wtime: Option<Duration>,
    /// Remaining time for black.
    pub btime: Option<Duration>,
    /// Time increment for white.
    pub winc: Option<Duration>,
    /// Time increment for black.
    pub binc: Option<Duration>,
    /// Moves until the next time control (cyclic time controls).
    pub movestogo: Option<u64>,
    /// Do not search beyond this depth.
    pub depth: Option<u64>,
    /// Do not search more than this many nodes.
    pub nodes: Option<u64>,
    /// Do not search further if mate in that many (or less) moves is found.
    pub mate: Option<u64>,
    /// Stop the search if it continues for longer than that.
    pub movetime: Option<Duration>,
    /// Do not exit if the search unless explicitly told to.
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
