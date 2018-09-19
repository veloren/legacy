// Standard
use std::sync::Arc;

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
    pub(crate) fn create_player(&mut self, alias: String, mode: PlayMode, po: Manager<ServerPostOffice>) -> EntityBuilder {
        match mode {
            PlayMode::Headless => self.world.create_entity(),
            PlayMode::Character => self.world.create_character(alias.clone()),
        }.with(Player {
            alias,
            mode,
        })
        .with(Client {
            postoffice: Arc::new(po)
        })
    }
}
