#![feature(nll)]

// Crates
#[macro_use]
extern crate log;

// Standard
use std::{io, sync::Arc};

// Library
use parking_lot::Mutex;
use syrup::Window;
use vek::*;

// Project
use client::{Client, ClientEvent, PlayMode};
use common::{
    audio::{AudioGen, Buffer, Stream},
    terrain::{chunk::ChunkContainer, VolOffs},
};

struct NoAudio {}
impl AudioGen for NoAudio {
    fn gen_stream(&self, id: u64, buffer: &Buffer, stream: &Stream) {}

    fn gen_buffer(&self, id: u64, buffer: &Buffer) {}

    fn drop_stream(&self, id: u64, buffer: &Buffer, stream: &Stream) {}

    fn drop_buffer(&self, id: u64, buffer: &Buffer) {}
}

struct Payloads {}
impl client::Payloads for Payloads {
    type Chunk = ();
    type Entity = ();
    type Audio = NoAudio;
}

fn gen_payload(_key: Vec3<VolOffs>, _con: Arc<Mutex<Option<ChunkContainer<<Payloads as client::Payloads>::Chunk>>>>) {}

fn drop_payload(_key: Vec3<VolOffs>, _con: Arc<ChunkContainer<<Payloads as client::Payloads>::Chunk>>) {}

fn main() {
    info!("Starting headless client...");

    let mut remote_addr = String::new();
    println!("Remote server address [127.0.0.1:59003]:");
    io::stdin().read_line(&mut remote_addr).unwrap();
    let mut remote_addr = remote_addr.trim();
    if remote_addr.is_empty() {
        remote_addr = "127.0.0.1:59003";
    } else if remote_addr == "m" {
        remote_addr = "91.67.21.222:38888";
    }

    let default_alias = common::util::names::generate();
    println!("Alias: [{}]", default_alias);
    let mut alias = String::new();
    io::stdin().read_line(&mut alias).unwrap();
    let mut alias = alias.trim().to_string();
    if alias.is_empty() {
        alias = default_alias.to_string();
    }

    let client = Client::<Payloads>::new(
        PlayMode::Headless,
        alias,
        &remote_addr.trim(),
        gen_payload,
        drop_payload,
        Arc::new(NoAudio {}),
        0,
    )
    .expect("error when attempting to initiate the client");

    let mut win = Window::initscr();
    win.writeln("Welcome to the Veloren headless client.");

    loop {
        for event in client.get_events() {
            match event {
                ClientEvent::RecvChatMsg { text } => win.writeln(text),
            }
        }

        if let Some(msg) = win.get() {
            client.send_chat_msg(msg.clone());
        }
    }
}
