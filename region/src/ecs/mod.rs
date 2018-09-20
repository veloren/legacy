// Modules
pub mod phys;
pub mod net;
pub mod character;
#[cfg(test)]
mod tests;

// Std
use std::collections::HashMap;

// Library
use specs::{
    World, Entity, Builder, EntityBuilder,
    saveload::MarkedBuilder,
};
use vek::*;

// Local
use self::phys::{Pos, Vel, Ori};
use self::net::{SyncMarker, SyncNode};
use self::character::{Character, Health};

pub trait CreateUtil {
    fn create_character(&mut self, name: String) -> EntityBuilder;
}

impl CreateUtil for World {
    fn create_character(&mut self, name: String) -> EntityBuilder {
        self.create_entity()
            .with(Pos(Vec3::zero()))
            .with(Vel(Vec3::zero()))
            .with(Ori(Quaternion::identity()))
            .with(Character {
                name,
            })
            .with(Health(100))
            .marked::<SyncMarker>()
    }
}

pub fn create_world() -> World {
    let mut world = World::new();

    // Net
    world.register::<SyncMarker>();
    world.add_resource(SyncNode {
        range: 0..1000000,
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
