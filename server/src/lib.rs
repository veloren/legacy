#![feature(integer_atomics)]
#![feature(duration_as_u128)]
#![feature(iterator_find_map)]

// Crates
pub extern crate specs;
extern crate vek;
extern crate parking_lot;
extern crate common;
extern crate region;

// Modules
pub mod api;
pub mod player;
pub mod net;
mod error;
mod tick;

// Reexports
pub use error::Error;

// Standard
use std::{
    sync::{Arc, atomic::{AtomicU64, Ordering}},
    net::{TcpListener, ToSocketAddrs},
    collections::HashMap,
    thread,
    time::Duration,
};

// Library
use specs::{World, Entity};
use parking_lot::RwLock;

// Project
use common::{
    manager::{Manager, Managed},
    msg::ServerPostOffice,
    Uid,
};
use region::{
    ecs, ecs::CreateUtil,
};

// Local
use net::{Client, DisconnectReason};
use api::Api;
use player::Player;

pub trait Payloads: Send + Sync + 'static {
    type Chunk: Send + Sync + 'static;
    type Entity: Send + Sync + 'static;
    type Client: Send + Sync + 'static;

    fn on_player_connect(&self, api: &dyn Api, player: Entity) {}
    fn on_player_disconnect(&self, api: &dyn Api, player: Entity, reason: DisconnectReason) {}
    fn on_chat_msg(&self, api: &dyn Api, player: Entity, text: &str) -> Option<String> {
        Some(format!("[{}] {}", api.world().read_storage::<Player>().get(player).map(|p| p.alias.as_str()).unwrap_or("<none"), text))
    }
}

pub struct Server<P: Payloads> {
    listener: Option<TcpListener>,
    time_ms: u64,
    world: World,
    payload: P,
}

// Wrapper

// We use this wrapper to pass `Server` around without locking it
pub struct Wrapper<S>(RwLock<S>);

impl<S> Wrapper<S> {
    pub fn do_for<R, F: FnOnce(&S) -> R>(&self, f: F) -> R {
        f(&self.0.read())
    }

    pub fn do_for_mut<R, F: FnOnce(&mut S) -> R>(&self, f: F) -> R {
        f(&mut self.0.write())
    }
}

impl<P: Payloads> Server<P> {
    pub fn new<S: ToSocketAddrs>(payload: P, bind_addr: S) -> Result<Manager<Wrapper<Self>>, Error> {

        let mut world = ecs::create_world();
        world.register::<Client>();
        world.register::<Player>();

        Ok(Manager::init(Wrapper(RwLock::new(Server {
            listener: Some(TcpListener::bind(bind_addr)?),
            time_ms: 0,
            world,
            payload,
        }))))
    }
}

impl<P: Payloads> Managed for Wrapper<Server<P>> {
    fn init_workers(&self, mgr: &mut Manager<Self>) {
        // Incoming clients worker
        Manager::add_worker(mgr, |srv, running, mut mgr| {
            let listener = srv.do_for_mut(|srv| srv
                .listener
                .take()
                .expect("Attempted to listen for clients on server without a listener")
            );

            while let Ok((stream, addr)) = listener.accept() {
                // Convert the incoming stream to a postoffice ready to begin the connection handshake
                if let Ok(po) = ServerPostOffice::to_client(stream) {
                    Manager::add_worker(&mut mgr, move |srv, running, mgr| {
                        if let Ok(client) = net::auth_client(srv, po) {
                            net::handle_player_post(srv, client, running, mgr);
                        }
                    });
                }
            }
        });
    }
}
