use std::time::Duration;

// Library
use vek::*;

// Project
use net::Message;
use util::post::{PostBox, PostOffice};

// SessionKind

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SessionKind {
    Connect,
    Disconnect,
    Ping,
}

impl Message for SessionKind {}

// CompStore

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CompStore {
    Pos(Vec3<f32>),
    Vel(Vec3<f32>),
    Dir(Vec2<f32>),
    Player { alias: String, mode: PlayMode },
    Character { name: String },
    Health(u32),
}

// ServerMsg

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ServerMsg {
    // SessionKind::Connect
    Connected {
        player_uid: Option<u64>,
        time: Duration,
    },

    // SessionKind::Disconnect
    Disconnect {
        reason: String,
    },

    // SessionKind::Ping
    Ping,

    // One-shot
    ChatMsg {
        text: String,
    },
    EntityDeleted {
        uid: u64,
    },
    CompUpdate {
        // This also acts as an EntityCreated message
        uid: u64,
        store: CompStore,
    },

    TimeUpdate(Duration),
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
    Connect {
        alias: String,
        mode: PlayMode,
    },

    // SessionKind::Disconnect
    Disconnect {
        reason: String,
    },

    // SessionKind::Ping
    Ping,

    // One-shot
    ChatMsg {
        text: String,
    },
    Cmd {
        args: Vec<String>,
    },
    PlayerEntityUpdate {
        pos: Vec3<f32>,
        vel: Vec3<f32>,
        dir: Vec2<f32>,
    },
}

impl Message for ClientMsg {}

pub type ServerPostOffice = PostOffice<SessionKind, ServerMsg, ClientMsg>;
pub type ClientPostOffice = PostOffice<SessionKind, ClientMsg, ServerMsg>;

pub type ServerPostBox = PostBox<SessionKind, ServerMsg, ClientMsg>;
pub type ClientPostBox = PostBox<SessionKind, ClientMsg, ServerMsg>;
