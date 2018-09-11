use std::fmt;

use common::net;

#[derive(Debug)]
pub enum Error {
    NetworkErr(net::Error),
}

impl From<net::Error> for Error {
    fn from(e: net::Error) -> Error { Error::NetworkErr(e) }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::NetworkErr(e) => write!(f, "{}", e),
        }
    }
}
