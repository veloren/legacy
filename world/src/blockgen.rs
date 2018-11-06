// Standard
use std::ops::{Add, Sub, Div, Mul, Neg, Rem};

// Library
use vek::*;
use noise::{NoiseFn, SuperSimplex, Seedable, HybridMulti, MultiFractal};

// Project
use common::terrain::chunk::Block;

// Local
use cachegen::CacheGen;
use overworldgen::{OverworldGen, Out as OverworldOut};
use treegen::TreeGen;
use towngen::TownGen;
use Gen;
use new_seed;

pub struct BlockGen {
    overworld_gen: CacheGen<OverworldGen, Vec2<i64>, OverworldOut>,
    tree_gen: TreeGen,
    town_gen: TownGen,
    warp_nz: HybridMulti,
}

impl BlockGen {
    pub fn new() -> Self {
        Self {
            overworld_gen: CacheGen::new(OverworldGen::new(), 4096),
            tree_gen: TreeGen::new(),
            town_gen: TownGen::new(),

            warp_nz: HybridMulti::new()
                .set_seed(new_seed())
                .set_octaves(3),
        }
    }

    pub fn get_invariant_z(&self, pos: Vec2<i64>) -> OverworldOut {
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

impl Gen<OverworldOut> for BlockGen {
    type In = Vec3<i64>;
    type Out = Block;

    fn sample<'a>(&'a self, pos: Vec3<i64>, overworld: &OverworldOut) -> Block {
        let pos_f64 = pos.map(|e| e as f64) * 1.0;

        let z_warp = self.get_warp(pos_f64, overworld.dry, overworld.land).mul(100.0);

        let town = self.town_gen.sample(pos, self.overworld_gen.internal());

        let z_alt = overworld.z_alt + z_warp - town.surface.map(|_| 1.0).unwrap_or(0.0);

        const GRASS_DEPTH: f64 = 3.5;

        if pos_f64.z < z_alt {
            if pos_f64.z < overworld.z_sea + 2.0 {
                Block::SAND
            } else if pos_f64.z < overworld.z_water - 1.0 {
                Block::EARTH
            } else if pos_f64.z > z_alt - GRASS_DEPTH {
                if let Some(surface_block) = town.surface {
                    surface_block
                } else if overworld.temp > 0.5 {
                    Block::gradient3(
                        Block::GRAD3_O_STONE,
                        Block::GRAD3_A_GRASS,
                        Block::GRAD3_B_SAND,
                        (overworld.temp.sub(0.65).mul(16.0))
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
                        ((1.0 - overworld.temp).sub(0.65).mul(16.0))
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
                let tree_block = self.tree_gen.sample(pos, self.overworld_gen.internal());

                None
                    .or(town.block)
                    .or(tree_block)
                    .unwrap_or(Block::AIR)
            }
        }
    }
}
