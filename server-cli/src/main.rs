// Crates
extern crate server;

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

    fn on_client_connect(&self, api: &Api, player: Entity) {
        println!("[INFO] {} connected", api.world().read_storage::<Player>().get(player).map(|p| p.alias.as_str()).unwrap_or("<none"));
    }

    fn on_client_disconnect(&self, api: &Api, player: Entity, reason: DisconnectReason) {
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
    Server::<Payloads>::new(Payloads, addr).expect("Could not start server");
}
