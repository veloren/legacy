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
use biomegen::BiomeGen;
use Gen;
use new_seed;

pub struct BlockGen {
    overworld_gen: CacheGen<OverworldGen>,
    biome_gen: CacheGen<BiomeGen>,
    warp_nz: HybridMulti,
}

impl BlockGen {
    pub fn new() -> Self {
        Self {
            overworld_gen: CacheGen::new(OverworldGen::new(), 4096),
            biome_gen: CacheGen::new(BiomeGen::new(), 4096),

            warp_nz: HybridMulti::new()
                .set_seed(new_seed())
                .set_octaves(4),
        }
    }

    fn get_warp(&self, pos: Vec3<f64>, dry: f64, land: f64) -> f64 {
        let scale = Vec3::new(
            512.0,
            512.0,
            512.0,
        );

        if dry > 0.25 && dry < 0.75 {
            self.warp_nz.get(pos.div(scale).into_array()).abs().mul(1.0 - dry.sub(0.5).abs().mul(4.0)).mul(land).max(0.0)
        } else {
            0.0
        }
    }
}

impl Gen for BlockGen {
    type In = Vec3<i64>;
    type Supp = ();
    type Out = Block;

    fn sample<'a>(&'a self, pos: Vec3<i64>, _: &()) -> Block {
        let pos_f64 = pos.map(|e| e as f64) * 1.0;

        let overworld = self.overworld_gen.sample(Vec2::from(pos), &());
        let biome = self.biome_gen.sample(Vec2::from(pos), &overworld);

        let z_warp = self.get_warp(pos_f64, overworld.dry, overworld.land).mul(128.0);

        let z_alt = overworld.z_alt + biome.z_hill + z_warp;

        if pos_f64.z < z_alt {
            if pos_f64.z < overworld.z_water - 1.0 {
                Block::EARTH
            } else {
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
                        ((256.0 - (overworld.z_alt - overworld.z_sea)).div(256.0).mul(64.0))
                            .max(0.0)
                            .min(64.0) as u8,
                    )
                } else {
                    Block::gradient3(
                        Block::GRAD3_O_STONE,
                        Block::GRAD3_A_GRASS,
                        Block::GRAD3_B_SNOW,
                        ((1.0 - overworld.temp).sub(0.4).mul(16.0))
                            .max(0.0)
                            .min(1.0)
                            .add(overworld.temp_vari)
                            .max(0.0)
                            .min(1.0)
                            .mul(32.0) as u8,
                        ((256.0 - (overworld.z_alt - overworld.z_sea)).div(256.0).mul(64.0))
                            .max(0.0)
                            .min(64.0) as u8,
                    )
                }
            }
        } else {
            if pos_f64.z < overworld.z_water {
                Block::WATER
            } else {
                Block::AIR
            }
        }
    }
}
