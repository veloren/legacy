// Standard
use std::fmt::{self, Display};
use std::sync::mpsc;

// Project
use common::net;

#[derive(Debug)]
pub enum Error {
    InvalidResponse,
    AlreadyRunning,
    MpscRecvErr(mpsc::RecvError),
    MpscRecvTimeoutErr(mpsc::RecvTimeoutError),
    MpscSendErr,
    NetworkErr(net::Error),
}

impl From<net::Error> for Error {
    fn from(e: net::Error) -> Error { Error::NetworkErr(e) }
}

impl From<mpsc::RecvError> for Error {
    fn from(e: mpsc::RecvError) -> Error { Error::MpscRecvErr(e) }
}

impl From<mpsc::RecvTimeoutError> for Error {
    fn from(e: mpsc::RecvTimeoutError) -> Error { Error::MpscRecvTimeoutErr(e) }
}

impl<T> From<mpsc::SendError<T>> for Error {
    fn from(e: mpsc::SendError<T>) -> Error { Error::MpscSendErr }
}
