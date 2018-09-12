// Crates
extern crate server;

// Project
use server::{Server, Player};

struct Payloads;
impl server::Payloads for Payloads {
    type Chunk = ();
    type Entity = ();
    type Player = ();

    fn on_player_connect(&self, player: &Player<Self::Player>) -> bool {
        println!("[{}] {} connected", player.get_uid(), player.alias());
        true
    }

    fn on_player_kick(&self, player: &Player<Self::Player>, reason: &str) -> bool {
        println!("[{}] {} kicked: {}", player.get_uid(), player.alias(), reason);
        true
    }

    fn on_player_disconnect(&self, player: &Player<Self::Player>) {
        println!("[{}] {} disconnected", player.get_uid(), player.alias());
    }

    fn on_chat_msg(&self, player: &Player<Self::Player>, text: &str) -> Option<String> {
        println!("[{}] {} : {}", player.get_uid(), player.alias(), text);
        Some(format!("[{}] {}", player.alias(), text))
    }
}

fn main() {
    let addr = "0.0.0.0:59003";
    println!("Starting server on {}...", addr);
    Server::<Payloads>::new(Payloads, addr).expect("Could not start server");
}
