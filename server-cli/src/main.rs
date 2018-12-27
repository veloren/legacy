extern crate clap;
use clap::{App, Arg};

// Project
use server::{api::Api, net::DisconnectReason, player::Player, specs::Entity, Manager, Server};

struct Payloads;
impl server::Payloads for Payloads {
    type Chunk = ();
    type Entity = ();
    type Client = ();

    fn on_player_connect(&self, api: &Api, player: Entity) {
        println!(
            "[INFO] {} connected",
            api.world()
                .read_storage::<Player>()
                .get(player)
                .map(|p| p.alias.as_str())
                .unwrap_or("<none")
        );

        api.send_chat_msg(player, "Welcome to the server! Type /help for more information");
    }

    fn on_player_disconnect(&self, api: &Api, player: Entity, reason: DisconnectReason) {
        println!(
            "[INFO] {} disconnected: {}",
            api.world()
                .read_storage::<Player>()
                .get(player)
                .map(|p| p.alias.as_str())
                .unwrap_or("<none"),
            reason
        );
    }

    fn on_chat_msg(&self, api: &Api, player: Entity, text: &str) -> Option<String> {
        let store = api.world().read_storage::<Player>();
        let alias = store.get(player).map(|p| p.alias.as_str()).unwrap_or("<none");
        println!("[CHAT] {}: {}", alias, text);
        Some(format!("{}: {}", alias, text))
    }
}

fn main() {
    let args = App::new("Veloren CLI server")
        .version(
            (option_env!("CARGO_PKG_VERSION").unwrap_or("UNKNOWN_VERSION").to_owned()
                + "."
                + option_env!("GIT_HASH").unwrap_or("UNKNOWN_GIT_HASH"))
            .as_str(),
        )
        .arg(
            Arg::with_name("addr")
                .short("a")
                .long("address")
                .value_name("ADDR")
                .help("Sets the listening address")
                .takes_value(true)
                .default_value("0.0.0.0"),
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .value_name("PORT")
                .help("Sets the listening port")
                .takes_value(true)
                .default_value("59003"),
        )
        .get_matches();
    let addr = args.value_of("addr").unwrap().to_owned() + ":" + args.value_of("port").unwrap(); //safe because of default_value
    println!("[INFO] Starting server on {}", addr);
    Manager::await_shutdown(Server::<Payloads>::new(Payloads, addr).expect("Could not start server"));
}
