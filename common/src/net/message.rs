// Standard
use std::{io, net::SocketAddr};

// Library
use bincode;
use serde::de::DeserializeOwned;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug)]
pub enum Error {
    NetworkErr(io::Error),
    CannotSerialize,
    CannotDeserialize,
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error { Error::NetworkErr(e) }
}

pub trait Message: Send + Sync + 'static + serde::Serialize + DeserializeOwned {
    fn to_bytes(&self) -> Result<Vec<u8>, Error> { bincode::serialize(&self).map_err(|_e| Error::CannotSerialize) }

    fn from_bytes(data: &[u8]) -> Result<Self, Error>
    where
        Self: Sized,
    {
        bincode::deserialize(data).map_err(|_e| Error::CannotDeserialize)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ConnectionMessage {
    OpenedUdp { host: SocketAddr },
    Shutdown,
    Ping,
}
impl Message for ConnectionMessage {}
