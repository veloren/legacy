// Library
use specs::{Component, VecStorage};
use vek::*;

// Character

#[derive(Debug)]
pub struct Character {
    pub name: String,
}

impl Component for Character {
    type Storage = VecStorage<Self>;
}

// Health

#[derive(Debug)]
pub struct Health(pub u32);

impl Component for Health {
    type Storage = VecStorage<Self>;
}
