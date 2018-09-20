// Standard
use std::{
    sync::Arc,
    time::Duration,
};

// Library
use specs::{Component, VecStorage, EntityBuilder, Builder};
use vek::*;

// Project
use region::ecs::CreateUtil;
use common::{
    manager::Manager,
    msg::{PlayMode, ServerPostOffice},
};

// Local
use Payloads;
use Server;
use net::Client;

// Player

#[derive(Debug)]
pub struct Player {
    pub alias: String,
    pub mode: PlayMode,
}

impl Component for Player {
    type Storage = VecStorage<Self>;
}

// Server

impl<P: Payloads> Server<P> {
    pub fn tick_once(&mut self, dt: Duration) {
        self.time_ms += dt.as_millis() as u64;

        // Sync entities with connected players
        self.sync_players();

        self.world.maintain();
    }
}
