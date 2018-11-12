#![feature(nll, euclidean_division, duration_as_u128, label_break_value)]

// Crates
extern crate common;
extern crate parking_lot;
extern crate vek;
extern crate world as world_crate; // TODO: Fix this naming conflict
#[macro_use]
extern crate log;

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
    terrain::{chunk::ChunkContainer, ChunkMgr, Entity, FnDropFunc, FnGenFunc, VolGen, VolOffs, VoxRel},
    util::{
        clock::Clock,
        manager::{Managed, Manager},
        msg::{ClientMsg, ClientPostOffice, ServerMsg, SessionKind},
    },
    Uid,
};

// Local
use error::Error;
use player::Player;

// Constants
pub const CHUNK_SIZE: Vec3<VoxRel> = Vec3 { x: 32, y: 32, z: 32 };
pub const CHUNK_MID: Vec3<f32> = Vec3 {
    x: CHUNK_SIZE.x as f32 / 2.0,
    y: CHUNK_SIZE.y as f32 / 2.0,
    z: CHUNK_SIZE.z as f32 / 2.0,
};
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

    clock: RwLock<Clock>,
    clock_tick_time: RwLock<Duration>,
    player: RwLock<Player>,
    entities: RwLock<HashMap<Uid, Arc<RwLock<Entity<<P as Payloads>::Entity>>>>>,
    phys_lock: Mutex<()>,

    chunk_mgr: ChunkMgr<<P as Payloads>::Chunk>,

    events: Mutex<Vec<ClientEvent>>,

    view_distance: i64,
}

impl<P: Payloads> Client<P> {
    pub fn new<
        S: ToSocketAddrs,
        GP: FnGenFunc<Vec3<VolOffs>, ChunkContainer<P::Chunk>>,
        DP: FnDropFunc<Vec3<VolOffs>, ChunkContainer<P::Chunk>>,
    >(
        mode: PlayMode,
        alias: String,
        remote_addr: S,
        gen_payload: GP,
        drop_payload: DP,
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

                clock: RwLock::new(Clock::new(Duration::from_millis(20))),
                clock_tick_time: RwLock::new(time),
                player: RwLock::new(Player::new(alias)),
                entities: RwLock::new(HashMap::new()),
                phys_lock: Mutex::new(()),

                chunk_mgr: ChunkMgr::new(
                    CHUNK_SIZE,
                    VolGen::new(world::gen_chunk, gen_payload, world::drop_chunk, drop_payload),
                ),

                events: Mutex::new(vec![]),

                view_distance: view_distance.max(CHUNK_SIZE.x as i64),
            });

            client.player.write().entity_uid = player_uid;

            Ok(client)
        } else {
            Err(Error::InvalidResponse)
        }
    }

    pub fn send_chat_msg(&self, text: String) { let _ = self.postoffice.send_one(ClientMsg::ChatMsg { text }); }

    pub fn send_cmd(&self, args: Vec<String>) { let _ = self.postoffice.send_one(ClientMsg::Cmd { args }); }

    pub fn view_distance(&self) -> f32 { self.view_distance as f32 }

    pub fn chunk_mgr(&self) -> &ChunkMgr<<P as Payloads>::Chunk> { &self.chunk_mgr }

    pub fn get_events(&self) -> Vec<ClientEvent> {
        let mut events = vec![];
        mem::swap(&mut events, &mut self.events.lock());
        events
    }

    pub fn status<'a>(&'a self) -> RwLockReadGuard<'a, ClientStatus> { self.status.read() }

    pub fn time(&self) -> Duration { *self.clock_tick_time.read() }

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
                let mut clocklock = client.clock.write();
                client.tick(clocklock.reference_duration(), &mut mgr);
                clocklock.tick();
                *client.clock_tick_time.write() += clocklock.reference_duration();
            }
        });

        // Chunkmgr worker
        Manager::add_worker(manager, |client, running, mut mgr| {
            let mut clock = Clock::new(Duration::from_millis(200));
            while running.load(Ordering::Relaxed) && *client.status() == ClientStatus::Connected {
                client.manage_chunks(&mut mgr);
                clock.tick();
            }
        });

        // Debug worker
        Manager::add_worker(manager, |client, running, mut mgr| {
            let mut clock = Clock::new(Duration::from_millis(5000));
            while running.load(Ordering::Relaxed) && *client.status() == ClientStatus::Connected {
                client.debug(&mut mgr);
                clock.tick();
            }
        });
    }

    fn on_drop(&self, _: &mut Manager<Self>) {
        *self.status.write() = ClientStatus::Disconnected;
        self.postoffice.stop();
    }
}
