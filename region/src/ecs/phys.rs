// Library
use specs::{Component, VecStorage};
use vek::*;

// Project
use common::msg::CompStore;

// Local
use super::NetComp;

// Pos

#[derive(Copy, Clone, Debug)]
pub struct Pos(pub Vec3<f32>);

impl Component for Pos {
    type Storage = VecStorage<Self>;
}

impl NetComp for Pos {
    fn to_store(&self) -> Option<CompStore> { Some(CompStore::Pos(self.0)) }
}

// Vel

#[derive(Copy, Clone, Debug)]
pub struct Vel(pub Vec3<f32>);

impl Component for Vel {
    type Storage = VecStorage<Self>;
}

impl NetComp for Vel {
    fn to_store(&self) -> Option<CompStore> { Some(CompStore::Vel(self.0)) }
}

// Dir

#[derive(Copy, Clone, Debug)]
pub struct Dir(pub Vec2<f32>);

impl Component for Dir {
    type Storage = VecStorage<Self>;
}

impl NetComp for Dir {
    fn to_store(&self) -> Option<CompStore> { Some(CompStore::Dir(self.0)) }
}
