// Parent
use super::{packet::Frame, Error};

pub const PROTOCOL_FRAME_HEADER: u8 = 1;
pub const PROTOCOL_FRAME_DATA: u8 = 2;

pub trait Protocol {
    fn send(&self, frame: Frame) -> Result<(), Error>;
    fn recv(&self) -> Result<Frame, Error>;
}
