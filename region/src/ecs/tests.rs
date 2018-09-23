// Library
use specs::{Builder, World};
use vek::*;

// Local
use super::*;

#[test]
fn test_create_raw_ecs() {
    use self::{
        character::{Character, Health},
        phys::{CtrlDir, Pos, Vel},
    };

    let mut world = World::new();
    world.register::<Pos>();
    world.register::<Vel>();
    world.register::<CtrlDir>();
    world.register::<Character>();
    world.register::<Health>();
    let e0 = world
        .create_entity()
        .with(Pos(Vec3::zero()))
        .with(Vel(Vec3::zero()))
        .with(CtrlDir(Vec2::zero()))
        .with(Character {
            name: "test".to_string(),
        }).with(Health(100))
        .build();
}

#[test]
fn test_create_world() {
    let mut world = create_world();

    let _c = world.create_character("wollay".to_string()).build();
}
