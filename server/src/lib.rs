// #![feature(nll, extern_prelude, box_syntax)]

// extern crate time;
// #[macro_use]
// extern crate coord;
// extern crate bifrost;
// extern crate serde;
// extern crate toml;
// #[macro_use]
// extern crate serde_derive;
// #[macro_use]
// extern crate log;
// #[macro_use]
// extern crate pretty_env_logger;
// extern crate common;
// extern crate region;
// extern crate world;

// mod config;
// mod init;
// mod network;
// mod player;
// pub mod server;
// mod server_context;
// mod session;

// pub use server::*;

#![feature(integer_atomics)]

// Crates
extern crate common;
extern crate region;

// Modules
mod player;
mod error;
mod net;

// Reexports
pub use player::Player;
pub use error::Error;

// Standard
use std::{
    sync::{Arc, RwLock, atomic::{AtomicU64, Ordering}},
    net::{TcpListener, ToSocketAddrs},
    collections::HashMap,
};

// Project
use common::{
    manager::{Manager, Managed},
    msg::ServerPostOffice,
    Uid,
};
use region::{
    Entity,
};

pub trait Payloads: Send + Sync + 'static {
    type Chunk: Send + Sync + 'static;
    type Entity: Send + Sync + 'static;
    type Player: Send + Sync + 'static;

    fn on_player_connect(&self, player: &Player<Self::Player>) -> bool { true }
    fn on_player_kick(&self, player: &Player<Self::Player>, reason: &str) -> bool { true }
    fn on_player_disconnect(&self, player: &Player<Self::Player>) {}
    fn on_chat_msg(&self, player: &Player<Self::Player>, text: &str) -> Option<String> {
        Some(format!("[{}] {}", player.alias(), text))
    }
}

pub struct Server<P: Payloads> {
    listener: TcpListener,

    payload: P,

    uid_counter: AtomicU64,
    time: RwLock<f64>,

    entities: RwLock<HashMap<Uid, Arc<Entity<<P as Payloads>::Entity>>>>,
    players: RwLock<HashMap<Uid, Arc<Player<<P as Payloads>::Player>>>>,
}

impl<P: Payloads> Server<P> {
    pub fn new<S: ToSocketAddrs>(payload: P, bind_addr: S) -> Result<Manager<Server<P>>, Error> {
        Ok(Manager::init(Server {
            listener: TcpListener::bind(bind_addr)?,

            payload,

            uid_counter: AtomicU64::new(0),
            time: RwLock::new(0.0),

            entities: RwLock::new(HashMap::new()),
            players: RwLock::new(HashMap::new()),
        }))
    }

    // Utility to generate a new unique UID
    fn gen_uid(&self) -> Uid { self.uid_counter.fetch_add(1, Ordering::Relaxed) as Uid }
}

impl<P: Payloads> Managed for Server<P> {
    fn init_workers(&self, mgr: &mut Manager<Self>) {
        // Incoming clients worker
        Manager::add_worker(mgr, |server, running, mut mgr| {
            while let Ok((stream, addr)) = server.listener.accept() {
                // Convert the incoming stream to a postoffice ready to begin the connection handshake
                if let Ok(po) = ServerPostOffice::to_client(stream) {
                    Manager::add_worker(&mut mgr, move |server, running, mgr| {
                        server.handle_incoming(po, running, mgr)
                    });
                }
            }
        });

        // Incoming clients worker
        Manager::add_worker(mgr, |server, running, mut mgr| {
            while let Ok((stream, addr)) = server.listener.accept() {
                // Convert the incoming stream to a postoffice ready to begin the connection handshake
                if let Ok(po) = ServerPostOffice::to_client(stream) {
                    Manager::add_worker(&mut mgr, move |server, running, mgr| {
                        server.handle_incoming(po, running, mgr)
                    });
                }
            }
        });
    }
}
