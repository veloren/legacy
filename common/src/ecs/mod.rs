// Modules
pub mod character;
pub mod net;
pub mod phys;
#[cfg(test)]
mod tests;

// Std
use std::collections::HashMap;

// Library
use crate::util::msg::CompStore;
use specs::{saveload::MarkedBuilder, Builder, Component, EntityBuilder, World};
use vek::*;

// Local
use self::{
    character::{Character, Health},
    net::{UidMarker, UidNode},
    phys::{Dir, Pos, Vel},
};

const MAX_UIDS: u64 = 1_000_000_000;

pub trait CreateUtil {
    fn create_character(&mut self, name: String) -> EntityBuilder;
}

impl CreateUtil for World {
    fn create_character(&mut self, name: String) -> EntityBuilder {
        self.create_entity()
            .with(Pos(Vec3::zero()))
            .with(Vel(Vec3::zero()))
            .with(Dir(Vec2::zero()))
            .with(Character { name })
            .with(Health(100))
            .marked::<UidMarker>()
    }
}

pub fn create_world() -> World {
    let mut world = World::new();

    // Net
    world.register::<UidMarker>();
    world.add_resource(UidNode {
        range: 0..MAX_UIDS,
        mapping: HashMap::new(),
    });
    // Phys
    world.register::<Pos>();
    world.register::<Vel>();
    world.register::<Dir>();
    // Character
    world.register::<Character>();
    world.register::<Health>();

    world
}

pub trait NetComp: Component {
    fn to_store(&self) -> Option<CompStore> { None }
}

// Default impl
impl<T> NetComp for T where T: Component {}
