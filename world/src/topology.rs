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
use overworld::{self, OverworldGen};

#[derive(Copy, Clone)]
pub struct Sample {
    pub block: Block,
}

pub struct TopologyGen {
    peak_nz: HybridMulti,
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
                .set_octaves(3),
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
        let scale = Vec3::new(300.0, 300.0, 200.0);
        let height = 60.0;
        self.peak_nz.get(pos.div(scale).into_array()).mul(chaos).mul(height)
    }

    // 0.0 = lowest, 1.0 = highest
    fn get_cliff(&self, pos: Vec3<f64>, dry: f64, chaos: f64) -> f64 {
        let scale = Vec3::new(512.0, 512.0, 2500.0);
        let spot_scale = Vec3::new(64.0, 64.0, 200.0);
        let layers = 4.0;

        (
            chaos.mul(0.3) +
            self.cliff_nz.get(pos.div(scale).into_array()).mul(chaos).mul(dry).mul(0.4) +
            self.cliff_spot_nz.get(pos.div(spot_scale).into_array()).max(0.0).mul(dry).mul(0.3)
        ).mul(layers).round().div(layers).max(0.0)
    }

    fn get_surf(&self, pos: Vec3<i64>) -> (overworld::Sample, f64, f64, f64, f64) {
        let overworld = self.overworld_gen.sample(Vec2::from(pos));

        let pos = pos.map(|e| e as f64);

        let peak = self.get_peak(pos, overworld.chaos);
        let cliff = self.get_cliff(pos, overworld.dry, overworld.chaos);

        let basic_surf = 50.0 + overworld.hill;
        let alt_surf = basic_surf - overworld.river + (cliff * overworld.cliff_height).max(peak + overworld.ridge);
        let water_surf = (basic_surf - 2.0).max(44.0);

        (overworld, basic_surf, peak, alt_surf, water_surf)
    }
}

impl Gen for TopologyGen {
    type In = Vec3<i64>;
    type Out = Sample;

    fn sample(&self, pos: Vec3<i64>) -> Sample {
        // Calculate the surface information
        let (overworld, basic_surf, peak, alt_surf, water_surf) = self.get_surf(pos);

        let pos = pos.map(|e| e as f64);

        // 0.0 = flat
        let surf_angle = peak - self.get_peak(pos - Vec3::unit_z(), overworld.chaos) - 1.0;

        // Work out which block we should be using
        let block = if pos.z < alt_surf - 8.0 {
            // Underground materials
            Block::new(BlockMaterial::Stone)
        } else if pos.z >= alt_surf {
            // Above-ground materials
            if pos.z < water_surf {
                Block::new(BlockMaterial::Water)
            } else {
                Block::new(BlockMaterial::Air)
            }
        } else if surf_angle < -0.5 {
            // Near-surface materials
            if pos.z < alt_surf - 3.5 {
                Block::new(BlockMaterial::Earth)
            } else {
                // Surface materials
                if pos.z < water_surf {
                    Block::new(BlockMaterial::Sand)
                } else if overworld.temp < -0.5 {
                    Block::new(BlockMaterial::Snow)
                } else if overworld.temp > 0.5 {
                    Block::new(BlockMaterial::Sand)
                } else {
                    Block::new(BlockMaterial::Grass)
                }
            }
        } else if pos.z < alt_surf - 5.0 {
            if surf_angle < -0.3 {
                Block::new(BlockMaterial::Earth)
            } else {
                Block::new(BlockMaterial::Stone)
            }
        } else {
            Block::new(BlockMaterial::Air)
        };

        Sample {
            block,
        }
    }
}
