// Library
use specs::{Component, VecStorage};

// Project
use common::msg::CompStore;


// Local
use super::NetComp;
use item::Item;

// Character

#[derive(Debug)]
pub struct Character {
    pub name: String,
}

// Inventory
// TODO: Move to a separate file when the game will need it 

#[derive(Debug)]
pub struct Inventory {
    pub slots: [[Item; 9]; 5],
}

impl Component for Inventory {
    type Storage = VecStorage<Self>;
}

impl NetComp for Inventory {
    fn to_store(&self) -> Option<CompStore> {
        Some(CompStore::Inventory) {
            slots: self.slots.clone(),
        }
    }
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
