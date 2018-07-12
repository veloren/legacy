#![feature(nll)]


#[macro_use]
extern crate conrod;

#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate gfx_device_gl;
extern crate glutin;
#[macro_use] extern crate enum_map;
extern crate nalgebra;
extern crate time;
extern crate chrono;
#[macro_use] extern crate coord;
extern crate dot_vox;
#[macro_use] extern crate toml;
#[macro_use] extern crate serde_derive;

extern crate client;
extern crate common;
extern crate region;

extern crate pretty_env_logger;
#[macro_use] extern crate log;

mod game;
mod window;
mod renderer;
mod mesh;
mod model_object;
mod pipeline;
mod camera;
mod render_volume;
mod keybinds;
mod key_state;
mod vox;
mod ui;
mod tests;

use std::io::{self, Write};

use chrono::{Utc, TimeZone, DateTime};

use client::ClientMode;
use game::Game;
use common::get_version;

// START Environment variables
const GIT_HASH: Option<&'static str> = option_env!("GIT_HASH");
const GIT_TIME: Option<&'static str> = option_env!("GIT_TIME");
const PROFILE: Option<&'static str> = option_env!("PROFILE");
const BUILD_TIME: Option<&'static str> = option_env!("BUILD_TIME");

pub fn get_git_hash() -> String {
    GIT_HASH.unwrap_or("UNKNOWN GIT HASH").to_string()
}

pub fn get_git_time() -> DateTime<Utc> {
    Utc.timestamp(GIT_TIME.unwrap_or("-1").to_string().parse().unwrap(), 0)
}

pub fn get_profile() -> String {
    PROFILE.unwrap_or("UNKNOWN PROFILE").to_string()
}

pub fn get_build_time() -> DateTime<Utc> {
    Utc.timestamp(BUILD_TIME.unwrap_or("-1").to_string().parse().unwrap(), 0)
}
// END Environment variables

fn main() {
    pretty_env_logger::init();

    info!("Starting Voxygen... Version: {}", get_version());

    let mut remote_addr = String::new();
    println!("Remote server address [Default: 127.0.0.1:59003] (use m for testserver):");

    let mut args = std::env::args();

    // expects single command line argument that is the remote_addr
    if args.len() == 2 {
        remote_addr = args.nth(1).expect("No argument");
    }
    else {
        // If args aren't correct then read from stdin
        print!("Enter address (blank for default): ");
        io::stdout().flush().expect("Failed to flush");
        io::stdin().read_line(&mut remote_addr).unwrap();
    }

    let mut remote_addr = remote_addr.trim();
    if remote_addr.len() == 0 {
        remote_addr = "127.0.0.1:59003";
    } else if remote_addr == "m" {
        remote_addr = "91.67.21.222:38888";
    }

    println!("Connecting to {}", remote_addr);

    Game::new(
        ClientMode::Character,
        common::names::generate(),
        remote_addr
    ).run();
}
