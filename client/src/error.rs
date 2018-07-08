use common::net;

#[derive(Debug)]
pub enum Error {
    NetworkErr(net::Error),
}

impl From<net::Error> for Error {
    fn from(e: net::Error) -> Error {
        Error::NetworkErr(e)
    }
}
