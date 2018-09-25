// Library
use specs::{Component, VecStorage};
use vek::*;

// Pos

#[derive(Debug)]
pub struct Pos(pub Vec3<f32>);

impl Component for Pos {
    type Storage = VecStorage<Self>;
}

// Vel

#[derive(Debug)]
pub struct Vel(pub Vec3<f32>);

impl Component for Vel {
    type Storage = VecStorage<Self>;
}

// CtrlDir

#[derive(Debug)]
pub struct CtrlDir(pub Vec2<f32>);

impl Component for CtrlDir {
    type Storage = VecStorage<Self>;
}
