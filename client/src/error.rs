// Standard
use std::sync::mpsc;

// Project
use common::net;

#[derive(Debug)]
pub enum Error {
    Unknown,
    AlreadyRunning,
    MpscErr(mpsc::RecvError),
    NetworkErr(net::Error),
}

impl From<net::Error> for Error {
    fn from(e: net::Error) -> Error { Error::NetworkErr(e) }
}

impl From<mpsc::RecvError> for Error {
    fn from(e: mpsc::RecvError) -> Error { Error::MpscErr(e) }
}
