// Crates
extern crate server;

// Standard
use std::{
    thread,
    time::Duration,
};

// Project
use server::{
    specs::Entity,
    Server,
    api::Api,
    net::DisconnectReason,
    player::Player,
};

struct Payloads;
impl server::Payloads for Payloads {
    type Chunk = ();
    type Entity = ();
    type Client = ();

    fn on_player_connect(&self, api: &Api, player: Entity) {
        println!("[INFO] {} connected", api.world().read_storage::<Player>().get(player).map(|p| p.alias.as_str()).unwrap_or("<none"));

        api.send_chat_msg(player, "Welcome to the server! Type /help for more information");
    }

    fn on_player_disconnect(&self, api: &Api, player: Entity, reason: DisconnectReason) {
        println!("[INFO] {} disconnected: {}", api.world().read_storage::<Player>().get(player).map(|p| p.alias.as_str()).unwrap_or("<none"), reason);
    }

    fn on_chat_msg(&self, api: &Api, player: Entity, text: &str) -> Option<String> {
        let store = api.world().read_storage::<Player>();
        let alias = store.get(player).map(|p| p.alias.as_str()).unwrap_or("<none");
        println!("[CHAT] {}: {}", alias, text);
        Some(format!("{}: {}", alias, text))
    }
}

fn main() {
    let addr = "0.0.0.0:59003";
    println!("[INFO] Starting server on {}", addr);
    let srv = Server::<Payloads>::new(Payloads, addr).expect("Could not start server");

    // Run the tick loop
    loop {
        let dt = Duration::from_millis(50);
        thread::sleep(dt);
        srv.do_for_mut(|srv| srv.tick_once(dt));
    }
}
