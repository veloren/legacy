// Library
use specs::{World, Builder};
use vek::*;

// Local
use super::*;

#[test]
fn test_create_raw_ecs() {
    use self::{
        phys::{Pos, Vel, Ori},
        character::{Character, Health},
        player::{Player, PlayerMode},
    };

    let mut world = World::new();
    world.register::<Pos>();
    world.register::<Vel>();
    world.register::<Ori>();
    world.register::<Character>();
    world.register::<Health>();
    let e0 = world.create_entity()
        .with(Pos(Vec3::zero()))
        .with(Vel(Vec3::zero()))
        .with(Ori(Quaternion::identity()))
        .with(Character {
            name: "test".to_string(),
        })
        .with(Health(100))
        .build();
}

#[test]
fn test_create_world() {
    let mut world = create_world();

    let _c = world.create_character("wollay".to_string()).build();
}
