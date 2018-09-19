// Modules
mod phys;
mod character;
#[cfg(test)]
mod tests;

// Library
use specs::{World, Entity, Builder, EntityBuilder};
use vek::*;

// Local
use self::phys::{Pos, Vel, Ori};
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
    }
}

pub fn create_world() -> World {
    let mut world = World::new();

    // Phys
    world.register::<Pos>();
    world.register::<Vel>();
    world.register::<Ori>();
    // Character
    world.register::<Character>();
    world.register::<Health>();

    world
}
