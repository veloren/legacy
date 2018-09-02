#![feature(nll)]

extern crate noise;
#[macro_use]
extern crate euler;
extern crate region;

mod gen;
mod map;

// Reexports
pub use map::{Biome, Map};

pub struct World {
    map: Map,
}

impl World {
    pub fn new(seed: u32, size: u32) -> World {
        World {
            map: Map::new(seed, size),
        }
    }

    pub fn tick(&mut self, secs: f64) { self.map.tick(secs); }

    #[allow(dead_code)]
    pub fn map(&mut self) -> &mut Map { &mut self.map }
}
