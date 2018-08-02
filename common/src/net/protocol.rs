// Parent
use super::{packet::Frame, Error};

pub trait Protocol {
    fn send(&self, frame: Frame) -> Result<(), Error>;
    fn recv(&self) -> Result<Frame, Error>;
}
