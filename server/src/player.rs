// Standard
use std::sync::Arc;

// Library
use specs::{Builder, Component, EntityBuilder, VecStorage};

// Project
use common::{
    manager::Manager,
    msg::{CompStore, PlayMode, ServerPostOffice},
};
use region::ecs::{CreateUtil, NetComp};

// Local
use net::Client;
use Payloads;
use Server;

// Player

#[derive(Clone, Debug)]
pub struct Player {
    pub alias: String,
    pub mode: PlayMode,
}

impl Component for Player {
    type Storage = VecStorage<Self>;
}

impl NetComp for Player {
    fn to_store(&self) -> Option<CompStore> {
        Some(CompStore::Player {
            alias: self.alias.clone(),
            mode: self.mode,
        })
    }
}

// Server

impl<P: Payloads> Server<P> {
    pub(crate) fn create_player(
        &mut self,
        alias: String,
        mode: PlayMode,
        po: Manager<ServerPostOffice>,
    ) -> EntityBuilder {
        match mode {
            PlayMode::Headless => self.world.create_entity(),
            PlayMode::Character => self.world.create_character(alias.clone()),
        }
        .with(Player { alias, mode })
        .with(Client {
            postoffice: Arc::new(po),
        })
    }
}
