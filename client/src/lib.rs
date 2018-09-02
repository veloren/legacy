#![feature(nll, euclidean_division)]

// Crates
#[macro_use]
extern crate log;
#[macro_use]
extern crate coord;
extern crate common;
extern crate parking_lot;
extern crate region;

// Modules
mod callbacks;
mod error;
mod net;
mod player;
mod tick;
mod world;

// Reexport
pub use common::net::ClientMode;

// Constants
pub const CHUNK_SIZE: i64 = 32;

// Standard
use std::{collections::HashMap, net::ToSocketAddrs, sync::Arc, thread, time};

// Library
use parking_lot::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

// Project
use common::{
    get_version,
    net::{Callback, ClientMessage, Connection, ServerMessage, UdpMgr},
    JobHandle, Jobs, Uid,
};
use region::{
    chunk::{Block, Chunk, ChunkContainer, ChunkConverter},
    Entity, FnPayloadFunc, VolGen, VolMgr, Volume, Voxel,
};

// Local
use callbacks::Callbacks;
use error::Error;
use player::Player;

#[derive(Copy, Clone, PartialEq)]
pub enum ClientStatus {
    Connecting,
    Connected,
    Timeout,
    Disconnected,
}

pub trait Payloads: 'static {
    type Chunk: Send + Sync + 'static;
    type Entity: Send + Sync + 'static;
}

pub struct Client<P: Payloads> {
    pub(crate) jobs: Jobs<Client<P>>,
    run_job: Mutex<Option<JobHandle<()>>>,
    run_job2: Mutex<Option<JobHandle<()>>>,

    status: RwLock<ClientStatus>,
    conn: Arc<Connection<ServerMessage>>,

    time: RwLock<f64>,
    player: RwLock<Player>,
    entities: RwLock<HashMap<Uid, Arc<RwLock<Entity<<P as Payloads>::Entity>>>>>,
    phys_lock: Mutex<()>,

    chunk_mgr: VolMgr<Chunk, ChunkContainer, ChunkConverter, <P as Payloads>::Chunk>,

    callbacks: RwLock<Callbacks>,

    view_distance: i64,
}

impl<P: Payloads> Callback<ServerMessage> for Client<P> {
    fn recv(&self, msg: Result<ServerMessage, common::net::Error>) { self.handle_packet(msg.unwrap()); }
}

impl<P: Payloads> Client<P> {
    pub fn new<U: ToSocketAddrs, GF: FnPayloadFunc<Chunk, ChunkContainer, P::Chunk>>(
        mode: ClientMode,
        alias: String,
        remote_addr: U,
        gen_payload: GF,
        view_distance: i64,
    ) -> Result<Arc<Client<P>>, Error> {
        let conn = Connection::new::<U>(&remote_addr, Box::new(|_m| {}), None, UdpMgr::new())?;
        conn.send(ClientMessage::Connect {
            mode,
            alias: alias.clone(),
            version: get_version(),
        });
        Connection::start(&conn);

        let client = Arc::new(Client {
            jobs: Jobs::new(),
            run_job: Mutex::new(None),
            run_job2: Mutex::new(None),

            status: RwLock::new(ClientStatus::Connecting),
            conn,

            time: RwLock::new(0.0),
            player: RwLock::new(Player::new(alias)),
            entities: RwLock::new(HashMap::new()),
            phys_lock: Mutex::new(()),

            chunk_mgr: VolMgr::new(CHUNK_SIZE, VolGen::new(world::gen_chunk, gen_payload)),

            callbacks: RwLock::new(Callbacks::new()),

            view_distance: view_distance.max(1).min(10),
        });

        *client.conn.callbackobj() = Some(client.clone());

        let client_ref = client.clone();
        client.jobs.set_root(client_ref);

        Ok(client)
    }

    fn set_status(&self, status: ClientStatus) { *self.status.write() = status; }

    pub fn start(&self) {
        let mut lock = self.run_job.lock();
        if lock.is_none() {
            *lock = Some(self.jobs.do_loop(|c| {
                thread::sleep(time::Duration::from_millis(40));
                c.tick(40.0 / 1000.0)
            }));
        }
        let mut lock = self.run_job2.lock();
        if lock.is_none() {
            *lock = Some(self.jobs.do_loop(|c| {
                thread::sleep(time::Duration::from_millis(1000));
                c.tick2(1.0)
            }));
        }
    }

    pub fn shutdown(&self) {
        self.conn.send(ClientMessage::Disconnect);
        self.set_status(ClientStatus::Disconnected);
        if let Some(jh) = self.run_job.lock().take() {
            jh.await();
        }
        if let Some(jh) = self.run_job2.lock().take() {
            jh.await();
        }
    }

    pub fn send_chat_msg(&self, msg: String) { self.conn.send(ClientMessage::ChatMsg { msg }) }

    pub fn send_cmd(&self, cmd: String) { self.conn.send(ClientMessage::SendCmd { cmd }) }

    pub fn view_distance(&self) -> f32 { (self.view_distance * CHUNK_SIZE) as f32 }

    pub fn chunk_mgr(&self) -> &VolMgr<Chunk, ChunkContainer, ChunkConverter, <P as Payloads>::Chunk> {
        &self.chunk_mgr
    }

    pub fn status<'a>(&'a self) -> RwLockReadGuard<'a, ClientStatus> { self.status.read() }

    pub fn callbacks<'a>(&'a self) -> RwLockReadGuard<'a, Callbacks> { self.callbacks.read() }

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
            .is_some()
    }

    pub fn player_entity(&self) -> Option<Arc<RwLock<Entity<<P as Payloads>::Entity>>>> {
        self.player().entity_uid.and_then(|uid| self.entity(uid))
    }
}
