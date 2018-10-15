#![feature(nll)]

extern crate common;
extern crate vek;
extern crate noise;
extern crate dot_vox;

mod util;

// Library
use noise::{NoiseFn, SuperSimplex, HybridMulti, Seedable, MultiFractal};
use vek::*;
use dot_vox::DotVoxData;

// Project
use common::terrain::{
    chunk::{Chunk, Block, BlockMaterial},
    Voxel,
};

const CHUNK_SZ: (u32, u32, u32) = (64, 64, 64);

pub struct World;

impl World {
    pub fn gen_chunk(offs: Vec3<i32>) -> Chunk {
        let chunk_sz = Vec3::from(CHUNK_SZ).map(|e: u32| e as i64);

        let mut voxels = Vec::new();

        for x in 0..CHUNK_SZ.0 {
            for y in 0..CHUNK_SZ.1 {
                for z in 0..CHUNK_SZ.2 {
                    voxels.push(Block::new(if z == 0 {
                        BlockMaterial::Stone
                    } else {
                        BlockMaterial::Air
                    }));
                }
            }
        }

        Chunk::new(
            chunk_sz,
            offs.map(|e| e as i64),
            voxels,
        )
    }
}
