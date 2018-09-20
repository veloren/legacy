#![feature(nll, euclidean_division)]

// Crates
#[macro_use]
extern crate log;
#[macro_use]
extern crate vek;
extern crate common;
extern crate region;

// Modules
mod callbacks;
mod error;
mod net;
mod player;
mod tick;
mod world;

// Reexport
pub use common::msg::PlayMode;
pub use region::{Block, Chunk, ChunkContainer, ChunkConverter, FnPayloadFunc, Volume, Voxel};

// Standard
use std::{
    collections::HashMap,
    net::ToSocketAddrs,
    sync::{Arc, Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard, atomic::Ordering},
    time::Duration,
    thread, time,
};

// Project
use common::{
    get_version,
    manager::{Manager, Managed},
    msg::{SessionKind, ClientMsg, ServerMsg, ClientPostOffice, ClientPostBox},
    Uid,
};
use region::{Entity, VolGen, VolMgr};

// Local
use callbacks::Callbacks;
use error::Error;
use player::Player;

// Constants
pub const CHUNK_SIZE: i64 = 32;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Copy, Clone, PartialEq)]
pub enum ClientStatus {
    Connected,
    Timeout,
    Disconnected,
}

pub trait Payloads: 'static {
    type Chunk: Send + Sync + 'static;
    type Entity: Send + Sync + 'static;
}

pub struct Client<P: Payloads> {
    status: RwLock<ClientStatus>,
    postoffice: Manager<ClientPostOffice>,

    time: RwLock<f64>,
    player: RwLock<Player>,
    entities: RwLock<HashMap<Uid, Arc<RwLock<Entity<<P as Payloads>::Entity>>>>>,
    phys_lock: Mutex<()>,

    chunk_mgr: VolMgr<Chunk, ChunkContainer<<P as Payloads>::Chunk>, ChunkConverter, <P as Payloads>::Chunk>,

    callbacks: RwLock<Callbacks>,

    view_distance: i64,
}

impl<P: Payloads> Client<P> {
    pub fn new<S: ToSocketAddrs, GF: FnPayloadFunc<Chunk, P::Chunk, Output = P::Chunk>>(
        mode: PlayMode,
        alias: String,
        remote_addr: S,
        gen_payload: GF,
        view_distance: i64,
    ) -> Result<Manager<Client<P>>, Error> {
        // Attempt to connect to the server
        let postoffice = ClientPostOffice::to_server(remote_addr)?;

        // Initiate a connection handshake
        let pb = postoffice.create_postbox(SessionKind::Connect);
        pb.send(ClientMsg::Connect { alias: alias.clone(), mode });

        // Was the handshake successful?
        if let ServerMsg::Connected { player_uid } = pb.recv_timeout(CONNECT_TIMEOUT)? {
            let client = Manager::init(Client {
                status: RwLock::new(ClientStatus::Connected),
                postoffice,

                time: RwLock::new(0.0),
                player: RwLock::new(Player::new(alias)),
                entities: RwLock::new(HashMap::new()),
                phys_lock: Mutex::new(()),

                chunk_mgr: VolMgr::new(CHUNK_SIZE, VolGen::new(world::gen_chunk, gen_payload)),

                callbacks: RwLock::new(Callbacks::new()),

                view_distance: view_distance.max(1).min(10),
            });

            client.player.write().unwrap().entity_uid = player_uid;

            Ok(client)
        } else {
            Err(Error::InvalidResponse)
        }
    }

    pub fn send_chat_msg(&self, text: String) {
        self.postoffice.send_one(ClientMsg::ChatMsg { text });
    }

    pub fn send_cmd(&self, args: Vec<String>) {
        self.postoffice.send_one(ClientMsg::Cmd { args });
    }

    pub fn view_distance(&self) -> f32 { (self.view_distance * CHUNK_SIZE) as f32 }

    pub fn chunk_mgr(
        &self,
    ) -> &VolMgr<Chunk, ChunkContainer<<P as Payloads>::Chunk>, ChunkConverter, <P as Payloads>::Chunk> {
        &self.chunk_mgr
    }

    pub fn status<'a>(&'a self) -> RwLockReadGuard<'a, ClientStatus> { self.status.read().unwrap() }

    pub fn callbacks<'a>(&'a self) -> RwLockReadGuard<'a, Callbacks> { self.callbacks.read().unwrap() }

    pub fn time(&self) -> f64 { *self.time.read().unwrap() }

    pub fn player<'a>(&'a self) -> RwLockReadGuard<'a, Player> { self.player.read().unwrap() }
    pub fn player_mut<'a>(&'a self) -> RwLockWriteGuard<'a, Player> { self.player.write().unwrap() }

    pub fn entities<'a>(&'a self) -> RwLockReadGuard<'a, HashMap<Uid, Arc<RwLock<Entity<<P as Payloads>::Entity>>>>> {
        self.entities.read().unwrap()
    }
    pub fn entities_mut<'a>(
        &'a self,
    ) -> RwLockWriteGuard<'a, HashMap<Uid, Arc<RwLock<Entity<<P as Payloads>::Entity>>>>> {
        self.entities.write().unwrap()
    }

    pub fn entity<'a>(&'a self, uid: Uid) -> Option<Arc<RwLock<Entity<<P as Payloads>::Entity>>>> {
        self.entities.read().unwrap().get(&uid).map(|e| e.clone())
    }

    pub fn take_phys_lock<'a>(&'a self) -> MutexGuard<'a, ()> { self.phys_lock.lock().unwrap() }

    pub fn add_entity(&self, uid: Uid, entity: Entity<<P as Payloads>::Entity>) -> bool {
        !self
            .entities
            .write()
            .unwrap()
            .insert(uid, Arc::new(RwLock::new(entity)))
            .is_some()
    }

    pub fn player_entity(&self) -> Option<Arc<RwLock<Entity<<P as Payloads>::Entity>>>> {
        self.player().entity_uid.and_then(|uid| self.entity(uid))
    }
}

impl<P: Payloads> Managed for Client<P> {
    fn init_workers(&self, manager: &mut Manager<Self>) {

        // Incoming messages worker
        Manager::add_worker(manager, |client, running, mut mgr| {
            while running.load(Ordering::Relaxed) && *client.status() == ClientStatus::Connected {
                client.handle_incoming(&mut mgr);
            }

            // Send a disconnect message to the server
            client.postoffice
                .create_postbox(SessionKind::Disconnect)
                .send(ClientMsg::Disconnect { reason: "Logging out".into() });
        });

        // Tick worker
        Manager::add_worker(manager, |client, running, mut mgr| {
            while running.load(Ordering::Relaxed) && *client.status() == ClientStatus::Connected {
                client.tick(40.0 / 1000.0, &mut mgr);
            }
        });
    }
}
