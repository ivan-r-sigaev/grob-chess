use game::Game;
use position::{LanMove, Position};
use std::{str::FromStr, time::Duration};

use crate::command::cursor::Cursor;

mod cursor;

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

    fn from_str(s: &str) -> Result<Self, Self::Err> {
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
            Some(fen) => Position::try_from_fen(&fen).ok(),
            None => Some(Position::initial_position()),
        };

        let mut maybe_game = maybe_position.map(Game::from_position);
        if let Some(ref mut game) = maybe_game {
            while let Some(lan_move) = self.next_token().and_then(|t| t.parse::<LanMove>().ok()) {
                let Some(chess_move) = game.position().lan_move(lan_move) else {
                    break;
                };
                if !game.try_make_move(chess_move) {
                    break;
                }
            }
        }

        let game = maybe_game.unwrap_or_else(|| Game::from_position(Position::initial_position()));
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
