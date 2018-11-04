// Standard
use std::ops::{Add, Sub, Div, Mul, Neg, Rem};

// Library
use vek::*;
use noise::{NoiseFn, SuperSimplex, Seedable, HybridMulti, MultiFractal};

// Local
use overworldgen::OverworldGen;
use Gen;
use new_seed;

pub struct BiomeGen {
    hill_nz: HybridMulti,
    cliff_nz: HybridMulti,
}

#[derive(Copy, Clone)]
pub struct Out {
    pub z_hill: f64,
    pub z_cliff_height: f64,
}

impl BiomeGen {
    pub fn new() -> Self {
        Self {
            hill_nz: HybridMulti::new()
                .set_seed(new_seed())
                .set_octaves(2),
            cliff_nz: HybridMulti::new()
                .set_seed(new_seed())
                .set_octaves(3),
        }
    }

    // 0 = lowest, 1 = highest
    fn get_hill(&self, pos: Vec2<f64>) -> f64 {
        let scale = 256.0;

        self.hill_nz.get(pos.div(scale).into_array()).add(1.0).div(2.0)
    }

    // 0 = lowest, 1 = highest
    fn get_cliff_height(&self, pos: Vec2<f64>, land: f64) -> f64 {
        let scale = 512.0;
        let vari = 0.2;
        let layers = 3.0;

        (land + self.cliff_nz.get(pos.div(scale).into_array()).mul(vari)).mul(layers).floor().div(layers)
    }
}

impl Gen for BiomeGen {
    type In = Vec2<i64>;
    type Supp = <OverworldGen as Gen>::Out;
    type Out = Out;

    fn sample<'a>(&'a self, pos: Vec2<i64>, overworld: &'a <OverworldGen as Gen>::Out) -> Out {
        let pos_f64 = pos.map(|e| e as f64) * 1.0;

        let hill = self.get_hill(pos_f64);
        let cliff_height = self.get_cliff_height(pos_f64, overworld.land);

        let z_hill = hill * 16.0 * overworld.dry * overworld.land.mul(5.0).min(1.0).max(0.0);

        let z_cliff_height = cliff_height * 64.0;

        Out {
            z_hill,
            z_cliff_height,
        }
    }
}
