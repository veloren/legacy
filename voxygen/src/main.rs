#![feature(nll)]

// Graphics
#[macro_use]
extern crate gfx;
extern crate conrod;
extern crate gfx_device_gl;
extern crate gfx_window_glutin;
extern crate glutin;

// Mathematics
#[macro_use]
extern crate coord;
extern crate alga;
extern crate nalgebra;

// File loading
extern crate dot_vox;
extern crate glsl_include;
extern crate toml;

// I/O
#[macro_use]
extern crate log;
extern crate pretty_env_logger;

// Utility
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate enum_map;
extern crate fnv;
extern crate indexmap;
extern crate parking_lot;

// Time
extern crate chrono;
extern crate time;

extern crate client;
extern crate common;
extern crate region;

mod camera;
mod game;
mod key_state;
mod keybinds;
mod tests;
mod ui;
mod window;

// Rendering
mod consts;
mod pipeline;
mod renderer;
mod shader;

// Pipelines
mod skybox;
mod tonemapper;
mod voxel;

use std::io::{self, Write};

use chrono::{DateTime, TimeZone, Utc};

use client::ClientMode;
use common::get_version;
use game::Game;

// START Environment variables
const GIT_HASH: Option<&'static str> = option_env!("GIT_HASH");
const GIT_TIME: Option<&'static str> = option_env!("GIT_TIME");
const PROFILE: Option<&'static str> = option_env!("PROFILE");
const BUILD_TIME: Option<&'static str> = option_env!("BUILD_TIME");

pub fn get_git_hash() -> &'static str { GIT_HASH.unwrap_or("UNKNOWN GIT HASH") }
pub fn get_git_time() -> DateTime<Utc> { Utc.timestamp(GIT_TIME.unwrap_or("-1").to_string().parse().unwrap(), 0) }
pub fn get_profile() -> &'static str { PROFILE.unwrap_or("UNKNOWN PROFILE") }

pub fn get_build_time() -> DateTime<Utc> { Utc.timestamp(BUILD_TIME.unwrap_or("-1").to_string().parse().unwrap(), 0) }
// END Environment variables

fn main() {
    pretty_env_logger::init();

    info!("Starting Voxygen... Version: {}", get_version());

    let mut args = std::env::args();
    let mut remote_addr = String::new();
    let mut remote_choice = String::new();

    // expects single command line argument that is the remote_addr
    if args.len() == 2 {
        remote_addr = args.nth(1).expect("No argument");
    } else {
        println!("");
        println!("Which server you want to connect to?");
        println!("    Press (1) to connect to the public veloren server (default)");
        println!("    Press (2) to connect to localhost");
        println!("    Press (3) to connect to another internet server");
        println!("");
        io::stdout().flush().expect("Failed to flush");
        io::stdin().read_line(&mut remote_choice).unwrap();
        let remote_choice = remote_choice.trim();
        if remote_choice == "2" {
            remote_addr = "127.0.0.1:59003".to_string();
        } else if remote_choice == "3" {
            // If args aren't correct then read from stdin
            print!("Enter address (e.g. 127.0.0.1:59003):");
            io::stdout().flush().expect("Failed to flush");
            io::stdin().read_line(&mut remote_addr).unwrap();
        } else {
            remote_addr = "91.67.21.222:38888".to_string();
        }

        remote_addr = remote_addr.trim().to_string();
    }

    println!("What name do you want to use?");
    let mut name_choice = String::new();
    io::stdout().flush().expect("Failed to flush");
    io::stdin().read_line(&mut name_choice).unwrap();
    let mut name_choice = name_choice.trim();
    if name_choice.len() == 0 {
        println!("No name chosen, generating random one...");
        name_choice = common::names::generate();
    }

    println!("");
    println!("What view distance do you want to use?");
    println!("For a smooth experience on slower hardware, we recommend 2.");
    println!("For faster computers, 10 is advised.");
    println!("If you experience lag, restart Veloren and change this setting again.");
    println!("");
    let mut view_distance_choice = String::new();
    io::stdout().flush().expect("Failed to flush");
    io::stdin().read_line(&mut view_distance_choice).unwrap();
    let view_distance = view_distance_choice.trim().parse::<i64>().unwrap_or_else(|_| {
        println!("Invalid input, defaulting to 4.");
        4
    });
    println!("using a view distance of {}.", view_distance);

    println!("Connecting to {}", remote_addr);

    Game::new(ClientMode::Character, name_choice, remote_addr, view_distance).run();
}
