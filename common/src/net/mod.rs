pub mod connection;
pub mod message;
mod packet;
mod protocol;
mod tcp;
#[cfg(test)]
mod tests;
mod udp;
pub mod udpmgr;

// Reexports
pub use self::{
    connection::{Callback, Connection},
    message::{ClientMessage, Error, Message, ServerMessage},
    udpmgr::UdpMgr,
};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum ClientMode {
    Headless,
    Character,
}
