//! Grob UCI
//!
//! This crate is the driver code for the UCI engine.

use crate::server::Server;

mod search;
mod server;
mod uci;
mod uci_cursor;

fn main() {
    Server::new().run();
}
