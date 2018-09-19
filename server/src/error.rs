// Standard
use std::io;

#[derive(Debug)]
pub enum Error {
    NoConnectSession,
    InvalidConnectSession,
    NoConnectMsg,
    IoErr(io::Error),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::IoErr(e)
    }
}
