#![feature(nll, euclidean_division, duration_as_u128)]

// Crates
extern crate common;
extern crate parking_lot;
extern crate vek;

// Modules
mod error;
mod net;
mod player;
mod tick;
mod world;

// Reexport
pub use common::util::msg::PlayMode;

// Standard
use std::{
    collections::HashMap,
    mem,
    net::ToSocketAddrs,
    sync::{atomic::Ordering, Arc},
    time::Duration,
};

// Library
use parking_lot::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};
use vek::*;

// Project
use common::{
    terrain::{
        chunk::{Chunk, ChunkContainer, ChunkConverter},
        Entity, FnPayloadFunc, VolGen, VolMgr,
    },
    util::{
        manager::{Managed, Manager},
        msg::{ClientMsg, ClientPostOffice, ServerMsg, SessionKind},
    },
    Uid,
};

// Local
use error::Error;
use player::Player;

// Constants
pub const CHUNK_SIZE: [i64; 3] = [32, 32, 32];
pub const CHUNK_MID: [f32; 3] = [
    CHUNK_SIZE[0] as f32 / 2.0,
    CHUNK_SIZE[1] as f32 / 2.0,
    CHUNK_SIZE[2] as f32 / 2.0,
];
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

pub enum ClientEvent {
    RecvChatMsg { text: String },
}

pub struct Client<P: Payloads> {
    status: RwLock<ClientStatus>,
    postoffice: Manager<ClientPostOffice>,

    time: RwLock<f64>,
    player: RwLock<Player>,
    entities: RwLock<HashMap<Uid, Arc<RwLock<Entity<<P as Payloads>::Entity>>>>>,
    phys_lock: Mutex<()>,

    chunk_mgr: VolMgr<Chunk, ChunkContainer, ChunkConverter, <P as Payloads>::Chunk>,

    events: Mutex<Vec<ClientEvent>>,

    view_distance: i64,
}

impl<P: Payloads> Client<P> {
    pub fn new<S: ToSocketAddrs, GF: FnPayloadFunc<Chunk, ChunkContainer, P::Chunk>>(
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
        let _ = pb.send(ClientMsg::Connect {
            alias: alias.clone(),
            mode,
        });

        // Was the handshake successful?
        if let ServerMsg::Connected { player_uid, time } = pb.recv_timeout(CONNECT_TIMEOUT)? {
            let client = Manager::init(Client {
                status: RwLock::new(ClientStatus::Connected),
                postoffice,

                time: RwLock::new(time),
                player: RwLock::new(Player::new(alias)),
                entities: RwLock::new(HashMap::new()),
                phys_lock: Mutex::new(()),

                chunk_mgr: VolMgr::new(
                    Vec3::from_slice(&CHUNK_SIZE),
                    VolGen::new(world::gen_chunk, gen_payload),
                ),

                events: Mutex::new(vec![]),

                view_distance: view_distance.max(1).min(10),
            });

            client.player.write().entity_uid = player_uid;

            Ok(client)
        } else {
            Err(Error::InvalidResponse)
        }
    }

    pub fn send_chat_msg(&self, text: String) { let _ = self.postoffice.send_one(ClientMsg::ChatMsg { text }); }

    pub fn send_cmd(&self, args: Vec<String>) { let _ = self.postoffice.send_one(ClientMsg::Cmd { args }); }

    pub fn view_distance(&self) -> f32 {
        (Vec3::from_slice(&CHUNK_SIZE).map(|e| e as f32) * (self.view_distance as f32)).magnitude()
    }

    pub fn chunk_mgr(&self) -> &VolMgr<Chunk, ChunkContainer, ChunkConverter, <P as Payloads>::Chunk> {
        &self.chunk_mgr
    }

    pub fn get_events(&self) -> Vec<ClientEvent> {
        let mut events = vec![];
        mem::swap(&mut events, &mut self.events.lock());
        events
    }

    pub fn status<'a>(&'a self) -> RwLockReadGuard<'a, ClientStatus> { self.status.read() }

    pub fn time(&self) -> f64 { *self.time.read() }

    pub fn player<'a>(&'a self) -> RwLockReadGuard<'a, Player> { self.player.read() }
    pub fn player_mut<'a>(&'a self) -> RwLockWriteGuard<'a, Player> { self.player.write() }

    pub fn entities<'a>(&'a self) -> RwLockReadGuard<'a, HashMap<Uid, Arc<RwLock<Entity<<P as Payloads>::Entity>>>>> {
        self.entities.read()
    }
    pub fn entities_mut<'a>(
        &'a self,
    ) -> RwLockWriteGuard<'a, HashMap<Uid, Arc<RwLock<Entity<<P as Payloads>::Entity>>>>> {
        self.entities.write()
    }

    pub fn entity<'a>(&'a self, uid: Uid) -> Option<Arc<RwLock<Entity<<P as Payloads>::Entity>>>> {
        self.entities.read().get(&uid).map(|e| e.clone())
    }

    pub fn take_phys_lock<'a>(&'a self) -> MutexGuard<'a, ()> { self.phys_lock.lock() }

    pub fn add_entity(&self, uid: Uid, entity: Entity<<P as Payloads>::Entity>) -> bool {
        !self
            .entities
            .write()
            .insert(uid, Arc::new(RwLock::new(entity)))
            .is_none()
    }

    pub fn remove_entity(&self, uid: Uid) -> bool { !self.entities.write().remove(&uid).is_some() }

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
            let _ = client
                .postoffice
                .create_postbox(SessionKind::Disconnect)
                .send(ClientMsg::Disconnect {
                    reason: "Logging out".into(),
                });
        });

        // Tick worker
        Manager::add_worker(manager, |client, running, mut mgr| {
            while running.load(Ordering::Relaxed) && *client.status() == ClientStatus::Connected {
                let dt = Duration::from_millis(50);
                client.tick(dt, &mut mgr);
            }
        });

        // Tick2 worker
        Manager::add_worker(manager, |client, running, mut mgr| {
            while running.load(Ordering::Relaxed) && *client.status() == ClientStatus::Connected {
                client.manage_chunks(500.0 / 1000.0, &mut mgr);
            }
        });
    }

    fn on_drop(&self, _: &mut Manager<Self>) {
        *self.status.write() = ClientStatus::Disconnected;
        self.postoffice.stop();
    }
}
