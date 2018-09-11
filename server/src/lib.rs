#![feature(nll, extern_prelude, box_syntax)]

extern crate time;
#[macro_use]
extern crate coord;
extern crate bifrost;
extern crate serde;
extern crate toml;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
#[macro_use]
extern crate pretty_env_logger;
extern crate common;
extern crate region;
extern crate world;

mod config;
mod error;
mod init;
mod network;
mod player;
pub mod server;
mod server_context;
mod session;

pub use server::*;
