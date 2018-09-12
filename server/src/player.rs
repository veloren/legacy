/*
// Project
use common::Uid;

pub struct Player {
    session_id: u32,
    uid: Uid,
    entity_uid: Option<Uid>,
    alias: String,
}

impl Player {
    pub fn new(session_id: u32, uid: Uid, entity_uid: Option<Uid>, alias: &str) -> Player {
        Player {
            session_id,
            uid,
            entity_uid,
            alias: alias.to_string(),
        }
    }

    pub fn alias<'a>(&'a self) -> &str { &self.alias }

    #[allow(dead_code)]
    pub fn get_uid(&self) -> Uid { self.uid }
    #[allow(dead_code)]
    pub fn get_session_id(&self) -> u32 { self.session_id }
    #[allow(dead_code)]
    pub fn get_entity_uid(&self) -> Option<Uid> { self.entity_uid }
}
*/

// Standard
use std::sync::Arc;

// Project
use common::{
    manager::{Manager, Managed},
    msg::{ServerPostOffice, ClientMode},
    Uid,
};

// Local
use Payloads;
use Server;

pub struct Player<P: Send + Sync + 'static> {
    uid: Uid,

    postoffice: Manager<ServerPostOffice>,

    alias: String,
    mode: ClientMode,
    payload: Option<P>,
}

impl<P: Send + Sync + 'static> Player<P> {
    pub fn new(uid: Uid, postoffice: Manager<ServerPostOffice>, alias: String, mode: ClientMode) -> Self {
        Self {
            uid,

            postoffice,

            alias,
            mode,
            payload: None,
        }
    }

    pub fn get_uid(&self) -> Uid { self.uid }
    pub fn get_mode(&self) -> ClientMode { self.mode }

    pub fn postoffice(&self) -> &Manager<ServerPostOffice> { &self.postoffice }
    pub fn alias(&self) -> &String { &self.alias }
    pub fn payload(&self) -> &Option<P> { &self.payload }
    pub fn payload_mut(&mut self) -> &mut Option<P> { &mut self.payload }
}

impl<P: Payloads> Server<P> {
    pub(crate) fn kick_player(&self, player_uid: Uid, reason: &str) {
        if let Some(player) = self.players.read().unwrap().get(&player_uid).map(|p| p.clone()) {
            if self.payload.on_player_kick(&player, reason) {
                self.broadcast_msg(&format!("[{} was kicked: {}]", player.alias(), reason));
                self.remove_player(player_uid);
            }
        }
    }

    pub(crate) fn remove_player(&self, player_uid: Uid) -> Option<Arc<Player<<P as Payloads>::Player>>> {
        self.players.write().unwrap().remove(&player_uid)
    }
}
