#![feature(nll)]

extern crate client;
extern crate common;
extern crate get_if_addrs;
extern crate pretty_env_logger;
extern crate syrup;
#[macro_use]
extern crate log;

use std::{io, process::exit, sync::mpsc};

use syrup::Window;

use client::{Chunk, Client, ClientEvent, PlayMode};

struct Payloads {}
impl client::Payloads for Payloads {
    type Chunk = ();
    type Entity = ();
}

fn gen_payload(_: &Chunk) -> <Payloads as client::Payloads>::Chunk { () }

fn main() {
    info!("Starting headless client...");

    let mut remote_addr = String::new();
    println!("Remote server address [127.0.0.1:59003]:");
    io::stdin().read_line(&mut remote_addr).unwrap();
    let mut remote_addr = remote_addr.trim();
    if remote_addr.len() == 0 {
        remote_addr = "127.0.0.1:59003";
    } else if remote_addr == "m" {
        remote_addr = "91.67.21.222:38888";
    }

    let default_alias = common::names::generate();
    println!("Alias: [{}]", default_alias);
    let mut alias = String::new();
    io::stdin().read_line(&mut alias).unwrap();
    let mut alias = alias.trim().to_string();
    if alias.len() == 0 {
        alias = default_alias.to_string();
    }

    let client = match Client::<Payloads>::new(PlayMode::Headless, alias, &remote_addr.trim(), gen_payload, 0) {
        Ok(c) => c,
        Err(e) => {
            println!("An error occured when attempting to initiate the client: {:?}", e);
            exit(0);
        },
    };

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
