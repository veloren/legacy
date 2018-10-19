// Standard
use std::ops::{Add, Sub, Div, Mul, Neg, Rem};

// Library
use vek::*;
use noise::{NoiseFn, SuperSimplex, HybridMulti, Seedable, MultiFractal};

// Project
use common::terrain::{
    Voxel,
    chunk::{Block, BlockMaterial},
};

// Local
use Gen;
use cachegen::CacheGen;
use overworld::OverworldGen;

#[derive(Copy, Clone)]
pub struct Sample {
    pub block: Block,
}

pub struct TopologyGen {
    peak_nz: HybridMulti,
    cliff_height_nz: SuperSimplex,
    cliff_spot_nz: SuperSimplex,
    cliff_nz: HybridMulti,
    overworld_gen: CacheGen<OverworldGen>,
}

impl TopologyGen {
    pub fn new() -> Self {
        let mut seed = 0;
        let mut new_seed = || { seed += 1; seed };

        Self {
            peak_nz: HybridMulti::new()
                .set_seed(new_seed())
                .set_octaves(2),
            cliff_height_nz: SuperSimplex::new()
                .set_seed(new_seed()),
            cliff_spot_nz: SuperSimplex::new()
                .set_seed(new_seed()),
            cliff_nz: HybridMulti::new()
                .set_seed(new_seed())
                .set_octaves(2),
            overworld_gen: OverworldGen::new(),
        }
    }

    // 0.0 = lowest, height = highest
    fn get_peak(&self, pos: Vec3<f64>, chaos: f64) -> f64 {
        let scale = 150.0;
        let height = 30.0;
        self.peak_nz.get(pos.div(scale).into_array()).mul(chaos).mul(height)
    }

    // (1.0 - vari) * height = lowest, 1.0 = avg, (1.0 + vari) * height = highest
    fn get_cliff_height(&self, pos: Vec3<f64>) -> f64 {
        let scale = 256.0;
        let vari = 0.3;
        let height = 100.0;

        self.cliff_height_nz.get(pos.div(scale).into_array()).mul(vari).add(1.0).mul(height)
    }

    // 0.0 = lowest, 1.0 = highest
    fn get_cliff(&self, pos: Vec3<f64>, dry: f64, chaos: f64) -> f64 {
        let scale = 512.0;
        let spot_scale = 128.0;
        let layers = 4.0;

        (
            chaos.mul(0.3) +
            self.cliff_nz.get(pos.div(scale).into_array()).mul(chaos).mul(dry).mul(0.4) +
            self.cliff_spot_nz.get(pos.div(spot_scale).into_array()).max(0.0).mul(dry).mul(0.3)
        ).mul(layers).round().div(layers).max(0.0)
    }
}

impl Gen for TopologyGen {
    type In = Vec3<i64>;
    type Out = Sample;

    fn sample(&self, pos: Vec3<i64>) -> Sample {
        let overworld = self.overworld_gen.sample(Vec2::from(pos));

        let pos = pos.map(|e| e as f64);

        let peak = self.get_peak(pos, overworld.chaos);
        let cliff_height = self.get_cliff_height(pos);
        let cliff = self.get_cliff(pos, overworld.dry, overworld.chaos);

        let basic_surf = 50.0 + overworld.hill;
        let alt_surf = basic_surf - overworld.river + (cliff * cliff_height).max(peak + overworld.ridge);
        let water_surf = (basic_surf - 2.0).max(44.0);

        let block = if pos.z < alt_surf - 6.0 {
            Block::new(BlockMaterial::Stone)
        } else if pos.z < alt_surf - 2.0 {
            Block::new(BlockMaterial::Earth)
        } else if pos.z < alt_surf {
            if pos.z < water_surf {
                Block::new(BlockMaterial::Sand)
            } else if overworld.temp < -0.5 {
                Block::new(BlockMaterial::Snow)
            } else if overworld.temp > 0.5 {
                Block::new(BlockMaterial::Sand)
            } else {
                Block::new(BlockMaterial::Grass)
            }
        } else if pos.z < water_surf {
            Block::new(BlockMaterial::Water)
        } else {
            Block::new(BlockMaterial::Air)
        };

        Sample {
            block,
        }
    }
}
