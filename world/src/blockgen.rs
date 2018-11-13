// Standard
use std::ops::{Div, Mul};

// Library
use noise::{HybridMulti, MultiFractal, NoiseFn, Seedable};
use vek::*;

// Project
use common::terrain::chunk::Block;

// Local
use cachegen::CacheGen;
use new_seed;
use overworldgen::{Out as OverworldOut, OverworldGen};
use towngen::{self, TownGen};
use Gen;

pub struct BlockGen {
    overworld_gen: CacheGen<OverworldGen, Vec2<i64>, OverworldOut>,
    town_gen: TownGen,
    warp_nz: HybridMulti,
}

impl BlockGen {
    pub fn new() -> Self {
        Self {
            overworld_gen: CacheGen::new(OverworldGen::new(), 4096),
            town_gen: TownGen::new(),

            warp_nz: HybridMulti::new().set_seed(new_seed()).set_octaves(3),
        }
    }

    pub fn get_invariant_z(&self, pos: Vec2<i64>) -> (OverworldOut, towngen::InvariantZ) {
        let overworld = self.overworld_gen.sample(pos, &());

        (
            overworld,
            self.town_gen
                .get_invariant_z(pos, (&overworld, &self.overworld_gen.internal())),
        )
    }

    fn get_warp(&self, pos: Vec3<f64>, dry: f64, land: f64) -> f64 {
        let scale = Vec3::new(350.0, 350.0, 350.0);

        self.warp_nz
            .get(pos.div(scale).into_array())
            .abs()
            .powf(0.5)
            .mul((dry - 0.2).min(land).min(0.4))
            .max(0.0)
    }
}

impl Gen<(OverworldOut, towngen::InvariantZ)> for BlockGen {
    type In = Vec3<i64>;
    type Out = Block;

    fn sample<'a>(
        &self,
        pos: Vec3<i64>,
        (overworld, towngen_invariant_z): &(OverworldOut, towngen::InvariantZ),
    ) -> Block {
        let pos_f64 = pos.map(|e| e as f64) * 1.0;

        let warp = self.get_warp(pos_f64, overworld.dry, overworld.land);
        let z_warp = warp.mul(96.0);

        let town = self
            .town_gen
            .sample(pos, &(towngen_invariant_z, overworld, self.overworld_gen.internal()));

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
                } else {
                    overworld.surface_block
                }
            } else {
                Block::STONE
            }
        } else {
            if pos_f64.z < overworld.z_water {
                Block::WATER
            } else {
                None.or(town.block).unwrap_or(Block::AIR)
            }
        }
    }
}
