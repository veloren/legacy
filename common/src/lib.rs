#![feature(nll, specialization, euclidean_division, integer_atomics, duration_float)]

extern crate bincode;
extern crate get_if_addrs;
#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate byteorder;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate enum_map;
extern crate num;
extern crate parking_lot;
extern crate rand;
extern crate specs;
extern crate threadpool;
extern crate time;
extern crate vek;
extern crate dot_vox;

pub mod ecs;
pub mod item;
pub mod net;
pub mod physics;
pub mod terrain;
pub mod util;

pub type Uid = u64;

const CARGO_VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

pub fn get_version() -> String { CARGO_VERSION.unwrap_or("UNKNOWN VERSION").to_string() }
