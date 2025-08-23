//! Engine
//!
//! This crate is the driver code for the engine.

use crate::server::Server;

mod search;
mod server;
mod uci;

fn main() {
    Server::new().run();
}
