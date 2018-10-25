// Standard
use std::ops::{Add, Sub, Div, Mul, Neg, Rem};

// Library
use vek::*;
use noise::{NoiseFn, SuperSimplex, HybridMulti, Seedable, MultiFractal};

// Project
use common::terrain::{
    Voxel,
    chunk::Block,
};

// Local
use Gen;
use cachegen::CacheGen;
use overworld::{self, OverworldGen};
use tree::{self, TreeGen};

#[derive(Copy, Clone)]
pub struct Sample {
    pub block: Block,
}

pub struct TopologyGen {
    peak_nz: HybridMulti,
    cliff_spot_nz: SuperSimplex,
    cliff_nz: HybridMulti,
    cave_nz: (HybridMulti, HybridMulti),
    overworld_gen: CacheGen<OverworldGen>,
    tree_gen: TreeGen,
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
            cave_nz: (
                HybridMulti::new()
                    .set_seed(new_seed())
                    .set_octaves(5),
                HybridMulti::new()
                    .set_seed(new_seed())
                    .set_octaves(5),
            ),
            overworld_gen: OverworldGen::new(),
            tree_gen: TreeGen::new(),
        }
    }

    pub fn overworld_gen(&self) -> &CacheGen<OverworldGen> {
        &self.overworld_gen
    }

    // 0.0 = lowest, height = highest
    fn get_peak(&self, pos: Vec3<f64>, chaos: f64) -> f64 {
        let scale = Vec3::new(250.0, 250.0, 300.0);
        let height = 65.0;
        self.peak_nz.get(pos.div(scale).into_array()).mul(chaos).mul(height)
    }

    // 0.0 = lowest, 1.0 = highest
    fn get_cliff(&self, pos: Vec3<f64>, dry: f64, chaos: f64) -> f64 {
        let scale = Vec3::new(220.0, 220.0, 1400.0);
        let spot_scale = Vec3::new(32.0, 32.0, 48.0);
        let layers = 4.0;

        (
            pos.z / 3000.0 +
            chaos.mul(0.3) +
            self.cliff_nz.get(pos.div(scale).into_array()).mul(chaos).mul(0.7) *
            (0.8 + self.cliff_spot_nz.get(pos.div(spot_scale).into_array()).mul(0.3))
        ).mul(layers).round().div(layers).max(0.0)
    }

    pub fn get_surf(&self, pos: Vec3<i64>) -> (overworld::Sample, f64, f64, f64, f64, f64, f64) {
        let overworld = self.overworld_gen.sample(Vec2::from(pos));

        let pos = pos.map(|e| e as f64);

        let basic_surf = 50.0 + overworld.hill;

        let peak = self.get_peak(pos, overworld.chaos);
        let mountain = peak + overworld.ridge;

        let cliff = self.get_cliff(pos, overworld.dry, overworld.chaos) * overworld.cliff_height;

        let alt_surf = basic_surf - overworld.river + cliff.max(mountain);
        let water_surf = (basic_surf - 0.1).min(48.0);

        (overworld, basic_surf, peak, cliff, mountain, alt_surf, water_surf)
    }
}

impl Gen for TopologyGen {
    type In = Vec3<i64>;
    type Out = Sample;

    fn sample(&self, pos: Vec3<i64>) -> Sample {
        // Calculate the surface information
        let (overworld, basic_surf, peak, cliff, mountain, alt_surf, water_surf) = self.get_surf(pos);
        let overworld_dx = self.overworld_gen.sample(Vec2::from(pos) + Vec2::unit_x() * 5);
        let overworld_dy = self.overworld_gen.sample(Vec2::from(pos) + Vec2::unit_y() * 5);

        let ridge_norm = Vec3::new(
            overworld_dx.ridge - overworld.ridge,
            overworld_dy.ridge - overworld.ridge,
            5.0,
        ).normalized();

        // Tree
        let tree = self.tree_gen.sample((pos, overworld, basic_surf));

        let pos = pos.map(|e| e as f64);

        // Work out which block we should be using
        let block = if let Some(tree_block) = tree.block {
            tree_block
        } else if pos.z >= alt_surf {
            // Above-ground materials
            if pos.z < water_surf {
                Block::WATER
            } else {
                Block::AIR
            }
        } else {
            let cave_scale = 800.0;
            let cave = false;
            //    self.cave_nz.0.get(pos.div(cave_scale).into_array()).abs() < 0.1 &&
            //    self.cave_nz.1.get(pos.div(cave_scale).into_array()).abs() < 0.1;

            if cave && pos.z < alt_surf - 8.0 {
                Block::AIR
            } else if pos.z < water_surf - 1.0 {
                Block::EARTH
            } else {
                if alt_surf - pos.z < 8.0 {
                    Block::gradient3(
                        if mountain > cliff + overworld.grad_vari * 4.0 {
                            Block::GRAD3_O_STONE
                        } else {
                            Block::GRAD3_O_EARTH
                        },
                        Block::GRAD3_A_GRASS,
                        Block::GRAD3_B_SAND,
                        (overworld.temp * 16.0 + overworld.grad_vari * 12.0)
                            .max(0.0)
                            .min(32.0) as u8,
                        (ridge_norm.z * 96.0 - 24.0)
                            .min((pos.z - (alt_surf - 6.0)) * 20.0)
                            .max(0.0)
                            .min(64.0) as u8,
                    )
                } else {
                    Block::gradient3(
                        Block::GRAD3_O_EARTH,
                        Block::GRAD3_A_LIMESTONE,
                        Block::GRAD3_B_SANDSTONE,
                        (16.0 + peak * 16.0)
                            .max(0.0)
                            .min(32.0) as u8,
                        ((alt_surf - pos.z - 8.0) * 8.0)
                            .max(0.0)
                            .min(64.0) as u8,
                    )
                }
            }
        };

        Sample {
            block,
        }
    }
}
