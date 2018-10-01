// Library
use specs::{Component, VecStorage};

// Project
use util::msg::CompStore;

// Local
use super::NetComp;

// Character

#[derive(Debug)]
pub struct Character {
    pub name: String,
}

impl Component for Character {
    type Storage = VecStorage<Self>;
}

impl NetComp for Character {
    fn to_store(&self) -> Option<CompStore> {
        Some(CompStore::Character {
            name: self.name.clone(),
        })
    }
}

// Health

#[derive(Debug)]
pub struct Health(pub u32);

impl Component for Health {
    type Storage = VecStorage<Self>;
}

impl NetComp for Health {
    fn to_store(&self) -> Option<CompStore> { Some(CompStore::Health(self.0)) }
}
