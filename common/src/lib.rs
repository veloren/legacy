#![feature(
    nll,
    specialization,
    euclidean_division,
    integer_atomics,
    duration_float,
    extern_crate_item_prelude
)]

#[macro_use]
extern crate log;

pub mod audio;
pub mod ecs;
pub mod item;
pub mod net;
pub mod physics;
pub mod terrain;
pub mod util;

// Standard
use std::path::{Path, PathBuf};

pub type Uid = u64;

const CARGO_VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

pub fn get_version() -> String { CARGO_VERSION.unwrap_or("UNKNOWN VERSION").to_string() }

pub fn get_asset_dir() -> &'static Path { Path::new(option_env!("VELOREN_ASSETS").unwrap_or("../assets/")) }

pub fn get_asset_path(rpath: &str) -> PathBuf { get_asset_dir().join(rpath) }
