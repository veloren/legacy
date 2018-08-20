// Standard
use std::{io, net::SocketAddr};

// Library
use bincode;
use coord::prelude::*;
use serde::{de::DeserializeOwned, Serialize};

// Local
use Uid;

// Parent
use super::ClientMode;

#[derive(Debug)]
pub enum Error {
    NetworkErr(io::Error),
    CannotSerialize,
    CannotDeserialize,
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error { Error::NetworkErr(e) }
}

pub trait Message: Send + Sync + 'static + Serialize + DeserializeOwned {
    fn to_bytes(&self) -> Result<Vec<u8>, Error> { bincode::serialize(&self).map_err(|_e| Error::CannotSerialize) }

    fn from_bytes(data: &[u8]) -> Result<Self, Error>
    where
        Self: Sized,
    {
        bincode::deserialize(data).map_err(|_e| Error::CannotDeserialize)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    Connected {
        entity_uid: Option<Uid>,
        version: String,
    },
    Kicked {
        reason: String,
    },
    Shutdown,
    Ping,
    Pong,
    RecvChatMsg {
        alias: String,
        msg: String,
    },
    EntityUpdate {
        uid: Uid,
        pos: Vec3f,
        vel: Vec3f,
        ctrl_acc: Vec3f,
        look_dir: Vec2f,
    },
    ChunkData {},
}
impl Message for ServerMessage {}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    Connect {
        mode: ClientMode,
        alias: String,
        version: String,
    },
    Disconnect,
    Ping,
    Pong,
    ChatMsg {
        msg: String,
    },
    SendCmd {
        cmd: String,
    },
    PlayerEntityUpdate {
        pos: Vec3f,
        vel: Vec3f,
        ctrl_acc: Vec3f,
        look_dir: Vec2f,
    },
}
impl Message for ClientMessage {}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ConnectionMessage {
    OpenedUdp { host: SocketAddr },
    Shutdown,
    Ping,
}
impl Message for ConnectionMessage {}
