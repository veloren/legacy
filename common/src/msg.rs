// Library
use vek::*;

// Project
use net::Message;
use post::{PostOffice, PostBox};

// SessionKind

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SessionKind {
    Connect,
    Disconnect,
    Ping,
}

impl Message for SessionKind {}

// ServerMsg

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ServerMsg {
    // SessionKind::Connect
    Connected { player_uid: Option<u64> },

    // SessionKind::Disconnect
    Disconnect { reason: String },

    // SessionKind::Ping
    Ping,

    // One-shot
    ChatMsg { text: String },
    EntityUpdate {
        uid: u64,
        pos: Vec3<f32>,
        vel: Vec3<f32>,
        ori: Quaternion<f32>,
    },
}

impl Message for ServerMsg {}

// ClientMsg

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum PlayMode {
    Headless,
    Character,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ClientMsg {
    // SessionKind::Connect
    Connect { alias: String, mode: PlayMode },

    // SessionKind::Disconnect
    Disconnect { reason: String },

    // SessionKind::Ping
    Ping,

    // One-shot
    ChatMsg { text: String },
    Cmd { args: Vec<String> },
    PlayerEntityUpdate {
        pos: Vec3<f32>,
        vel: Vec3<f32>,
        ori: Quaternion<f32>,
    },
}

impl Message for ClientMsg {}

pub type ServerPostOffice = PostOffice<SessionKind, ServerMsg, ClientMsg>;
pub type ClientPostOffice = PostOffice<SessionKind, ClientMsg, ServerMsg>;

pub type ServerPostBox = PostBox<SessionKind, ServerMsg, ClientMsg>;
pub type ClientPostBox = PostBox<SessionKind, ClientMsg, ServerMsg>;
