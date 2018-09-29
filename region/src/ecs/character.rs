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
    pub slots: [[Option<Item>; 9]; 5],
}

impl Component for Inventory {
    type Storage = VecStorage<Self>;
}

impl NetComp for Inventory {
    fn to_store(&self) -> Option<CompStore> {
        Some(CompStore::Inventory {
            slots: self.slots.clone(),
        })
    }
}

impl Inventory {
    pub fn get(&self, x: usize, y: usize) -> Option<Item> {
        self.slots
        .get(x)
        .and_then(|column| column.get(y))
        .and_then(|cell| *cell) 
    }

    pub fn give(&mut self, x: usize, y: usize, item: Item) -> Option<Item> {
        self.slots
        .get(x)
        .and_then(|column| column.get(y))
        .and_then(|cell| cell.replace(item))
    } 

    pub fn take(&mut self, x: usize, y: usize) -> Option<Item> {
        self.slots
        .get(x)
        .and_then(|column| column.get(y))
        .and_then(|cell| cell.take())
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
