//! Engine
//!
//! This crate is the driver code for the engine.

use crate::server::Server;

mod uci;
mod search;
mod server;

fn main() {
    Server::new().run();
}
