// Standard
use std::time::Duration;

// Library
use bifrost::{event, Relay};
use config::load_config;

// Project
//use common::logger::ConsoleLogger;

// Local
use network::init::init_network;
use server_context::{update_sessions_list, update_world, ServerContext, SESSION_UPDATE_TICK, WORLD_UPDATE_TICK};

pub fn init_server(relay: &Relay<ServerContext>, ctx: &mut ServerContext) {
    let config = load_config();

    init_network(relay.clone(), ctx, config.network.port);

    relay.schedule(event(update_world), Duration::from_millis(WORLD_UPDATE_TICK));
    relay.schedule(event(update_sessions_list), Duration::from_millis(SESSION_UPDATE_TICK));

    info!("Server started");
}
