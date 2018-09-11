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
extern crate pretty_env_logger;
extern crate region;

// Modules
mod player;
mod error;

// Standard
use std::{
    sync::{Arc, RwLock, atomic::{AtomicU64, Ordering}},
    net::{TcpListener, ToSocketAddrs},
    collections::HashMap,
};

// Project
use common::{
    manager::{Manager, Managed},
    msg::{SessionKind, ClientMsg, ServerMsg, ServerPostOffice, ServerPostBox},
    Uid,
};
use region::{
    Entity,
};

// Local
use player::Player;
use error::Error;

pub trait Payloads: 'static {
    type Chunk: Send + Sync + 'static;
    type Entity: Send + Sync + 'static;
    type Player: Send + Sync + 'static;
}

pub struct Server<P: Payloads> {
    listener: TcpListener,

    uid_counter: AtomicU64,
    time: RwLock<f64>,

    entities: RwLock<HashMap<Uid, Arc<RwLock<Entity<<P as Payloads>::Entity>>>>>,
    players: RwLock<HashMap<Uid, Arc<RwLock<Player>>>>,
}

impl<P: Payloads> Server<P> {
    pub fn new<S: ToSocketAddrs>(bind_addr: S) -> Result<Manager<Server<P>>, Error> {
        Ok(Manager::init(Server {
            listener: TcpListener::bind(bind_addr)?,

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
    fn init_workers(&self, manager: &mut Manager<Self>) {
        // Incoming clients worker
        Manager::add_worker(manager, |server, running, _| {
            while let Ok((stream, addr)) = server.listener.accept() {
                if let Ok(po) = ServerPostOffice::to_client(stream) {
                    let uid = server.gen_uid();

                    let player = Arc::new(RwLock::new(po.into()));

                    server.players.write().unwrap().insert(uid, player);
                    println!("Incoming!");
                }
            }
        });
    }
}
