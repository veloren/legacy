// Standard
use std::ops::{Add, Sub, Div, Mul, Neg, Rem};

// Library
use vek::*;
use noise::{NoiseFn, SuperSimplex, Seedable, HybridMulti, MultiFractal};

// Project
use common::terrain::chunk::Block;

// Local
use cachegen::CacheGen;
use overworldgen::OverworldGen;
use treegen::TreeGen;
use Gen;
use new_seed;

pub struct BlockGen {
    overworld_gen: CacheGen<OverworldGen>,
    tree_gen: TreeGen,
    warp_nz: HybridMulti,
}

impl BlockGen {
    pub fn new() -> Self {
        Self {
            overworld_gen: CacheGen::new(OverworldGen::new(), 4096),
            tree_gen: TreeGen::new(),

            warp_nz: HybridMulti::new()
                .set_seed(new_seed())
                .set_octaves(6),
        }
    }

    pub fn get_invariant_z(&self, pos: Vec2<i64>) -> <Self as Gen>::Supp {
        self.overworld_gen.sample(pos, &())
    }

    fn get_warp(&self, pos: Vec3<f64>, dry: f64, land: f64) -> f64 {
        let scale = Vec3::new(
            400.0,
            400.0,
            300.0,
        );

        if dry > 0.15 && dry < 0.85 {
            self.warp_nz.get(pos.div(scale).into_array()).abs().mul(1.0 - dry.sub(0.5).abs().mul(2.0 / 0.7)).mul(land).max(0.0)
        } else {
            0.0
        }
    }
}

impl Gen for BlockGen {
    type In = Vec3<i64>;
    type Supp = <OverworldGen as Gen>::Out;
    type Out = Block;

    fn sample<'a>(&'a self, pos: Vec3<i64>, overworld: &Self::Supp) -> Block {
        let pos_f64 = pos.map(|e| e as f64) * 1.0;

        let z_warp = self.get_warp(pos_f64, overworld.dry, overworld.land).mul(100.0);

        let z_alt = overworld.z_alt + z_warp;

        const GRASS_DEPTH: f64 = 3.5;

        if pos_f64.z < z_alt {
            if pos_f64.z < overworld.z_water - 1.0 {
                Block::EARTH
            } else if pos_f64.z > z_alt - GRASS_DEPTH {
                if overworld.temp > 0.0 {
                    Block::gradient3(
                        Block::GRAD3_O_STONE,
                        Block::GRAD3_A_GRASS,
                        Block::GRAD3_B_SAND,
                        (overworld.temp.sub(0.4).mul(16.0))
                            .max(0.0)
                            .min(1.0)
                            .add(overworld.temp_vari)
                            .max(0.0)
                            .min(1.0)
                            .mul(32.0) as u8,
                        ((256.0 - (overworld.z_alt - overworld.z_sea)).div(256.0)).max(0.0)
                            .min(1.0)
                            .add(overworld.alt_vari)
                            .max(0.0)
                            .min(1.0)
                            .mul(64.0) as u8,
                    )
                } else {
                    Block::gradient3(
                        Block::GRAD3_O_STONE,
                        Block::GRAD3_A_GRASS,
                        Block::GRAD3_B_SNOW,
                        ((-overworld.temp).sub(0.4).mul(16.0))
                            .max(0.0)
                            .min(1.0)
                            .add(overworld.temp_vari)
                            .max(0.0)
                            .min(1.0)
                            .mul(32.0) as u8,
                        ((256.0 - (overworld.z_alt - overworld.z_sea)).div(256.0)).max(0.0)
                            .min(1.0)
                            .add(overworld.alt_vari)
                            .max(0.0)
                            .min(1.0)
                            .mul(64.0) as u8,
                    )
                }
            } else {
                Block::STONE
            }
        } else {
            if pos_f64.z < overworld.z_water {
                Block::WATER
            } else {
                self.tree_gen.sample(pos, self.overworld_gen.internal()).unwrap_or(Block::AIR)
            }
        }
    }
}
