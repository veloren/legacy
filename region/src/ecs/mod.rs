// Modules
pub mod character;
pub mod net;
pub mod phys;
#[cfg(test)]
mod tests;

// Std
use std::collections::HashMap;

// Library
use specs::{saveload::MarkedBuilder, Builder, Entity, EntityBuilder, World};
use vek::*;

// Local
use self::{
    character::{Character, Health},
    net::{SyncMarker, SyncNode},
    phys::{Ori, Pos, Vel},
};

pub trait CreateUtil {
    fn create_character(&mut self, name: String) -> EntityBuilder;
}

impl CreateUtil for World {
    fn create_character(&mut self, name: String) -> EntityBuilder {
        self.create_entity()
            .with(Pos(Vec3::zero()))
            .with(Vel(Vec3::zero()))
            .with(Ori(Quaternion::identity()))
            .with(Character { name })
            .with(Health(100))
            .marked::<SyncMarker>()
    }
}

pub fn create_world() -> World {
    let mut world = World::new();

    // Net
    world.register::<SyncMarker>();
    world.add_resource(SyncNode {
        range: 0..1_000_000,
        mapping: HashMap::new(),
    });
    // Phys
    world.register::<Pos>();
    world.register::<Vel>();
    world.register::<Ori>();
    // Character
    world.register::<Character>();
    world.register::<Health>();

    world
}
