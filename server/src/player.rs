// Standard
use std::sync::Arc;

// Library
use specs::{Builder, Component, Entity, EntityBuilder, VecStorage};
use vek::*;

// Project
use common::{
    manager::Manager,
    msg::{PlayMode, ServerPostOffice},
};
use region::ecs::{
    phys::{Ori, Pos, Vel},
    CreateUtil,
};

// Local
use net::Client;
use Payloads;
use Server;

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
    pub(crate) fn create_player(
        &mut self,
        alias: String,
        mode: PlayMode,
        po: Manager<ServerPostOffice>,
    ) -> EntityBuilder {
        match mode {
            PlayMode::Headless => self.world.create_entity(),
            PlayMode::Character => self.world.create_character(alias.clone()),
        }.with(Player { alias, mode })
        .with(Client {
            postoffice: Arc::new(po),
        })
    }

    pub(crate) fn update_player_entity(
        &mut self,
        player: Entity,
        pos: Vec3<f32>,
        vel: Vec3<f32>,
        ori: Quaternion<f32>,
    ) {
        self.world.write_storage::<Pos>().get_mut(player).map(|p| {
            if Vec2::<f32>::from(p.0).distance(pos.into()) < 3.0 {
                p.0 = pos
            }
        }); // Basic sanity check
        self.world.write_storage::<Vel>().get_mut(player).map(|v| v.0 = vel);
        self.world.write_storage::<Ori>().get_mut(player).map(|o| o.0 = ori);
    }
}
