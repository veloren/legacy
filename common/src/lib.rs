#![feature(nll)]

extern crate bincode;
extern crate coord;
extern crate get_if_addrs;
#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate byteorder;
extern crate parking_lot;
extern crate rand;
extern crate time;

// Silence a compiler warning about macro_use not being used.
#[cfg(test)]
#[macro_use]
extern crate lazy_static;
#[cfg(not(test))]
extern crate lazy_static;

// Reexports
pub use clock::Clock;

pub mod clock;
pub mod jobs;
pub mod names;
pub mod net;

pub use jobs::{JobHandle, JobMultiHandle, Jobs};

pub type Uid = u64;

const CARGO_VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

pub fn get_version() -> String { CARGO_VERSION.unwrap_or("UNKNOWN VERSION").to_string() }
