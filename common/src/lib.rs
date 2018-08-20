#![feature(nll, integer_atomics)]

extern crate bincode;
extern crate get_if_addrs;
#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate byteorder;
extern crate coord;
extern crate rand;
extern crate time;
#[macro_use]
extern crate lazy_static;
extern crate parking_lot;

// Reexports
pub use clock::Clock;

pub mod clock;
pub mod names;
//pub mod network;
pub mod jobs;
pub mod net;
pub mod post;
pub mod session;

pub use jobs::{JobHandle, JobMultiHandle, Jobs};

pub type Uid = u64;

const CARGO_VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

pub fn get_version() -> String { CARGO_VERSION.unwrap_or("UNKNOWN VERSION").to_string() }
