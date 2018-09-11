
// Crates
extern crate pretty_env_logger;
#[macro_use]
extern crate log;
extern crate server;

// Project
use server::Server;

struct Payloads {}
impl server::Payloads for Payloads {
    type Chunk = ();
    type Entity = ();
    type Player = ();
}

fn main() {
    pretty_env_logger::init();
    info!("Starting server...");

    Server::<Payloads>::new("0.0.0.0:59003");
}
