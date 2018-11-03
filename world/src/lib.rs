#![feature(nll, fn_traits, associated_type_defaults, self_struct_ctor, euclidean_division)]

extern crate common;
extern crate vek;
extern crate noise;
extern crate dot_vox;
extern crate num_traits;
#[macro_use]
extern crate lazy_static;
extern crate fnv;
extern crate parking_lot;

mod util;
mod cachegen;
mod overworld;
mod topology;
mod tree;

// Standard
use std::{mem, hash::Hash};

// Library
use vek::*;

// Project
use common::terrain::{
    Volume,
    Voxel,
    chunk::{
        Chunk,
        Block,
        HeterogeneousData,
        HomogeneousData,
    },
    ConstructVolume,
    ReadWriteVolume,
};

// Local
use overworld::OverworldGen;
use topology::TopologyGen;

// Generator

pub trait Gen {
    type In: Clone;
    type Out: Clone;

    fn sample(&self, i: Self::In) -> Self::Out;
}

// World

const CHUNK_SZ: Vec3<u32> = Vec3 {
    x: 64,
    y: 64,
    z: 64,
};

lazy_static! {
    static ref GENERATOR: TopologyGen = TopologyGen::new();
}

pub struct World;

impl World {
    pub fn gen_chunk(offs: Vec3<i32>) -> Chunk {
        // If the chunk is out of bounds, just generate air
        if offs.z < 0 || offs.z > 8 {
            return Chunk::Homo(
                HomogeneousData::filled(CHUNK_SZ, Block::AIR)
            );
        }

        let mut chunk_data = HeterogeneousData::empty(CHUNK_SZ);

        // is_homogeneous, block type
        let mut cblock = (true, None);

        let mut gen_block_fn = |x, y, z| {
            let pos = offs.map(|e| e as i64) * CHUNK_SZ.map(|e| e as i64) + Vec3::new(x, y, z).map(|e| e as i64);

            let block = GENERATOR.sample(pos).block;

            match cblock {
                (true, None) => cblock.1 = Some(block),
                (true, Some(b)) if b == block => {},
                (true, Some(b)) => cblock = (false, None),
                _ => {},
            }

            chunk_data.set_at(
                Vec3::new(x, y, z),
                block,
            );
        };

        // x faces

        for x in (0..CHUNK_SZ.x).step_by(CHUNK_SZ.x as usize - 1) {
            for y in 1..CHUNK_SZ.y - 1 {
                for z in 1..CHUNK_SZ.z - 1 {
                    gen_block_fn(x, y, z);
                }
            }
        }

        // y faces

        for x in 0..CHUNK_SZ.x {
            for y in (0..CHUNK_SZ.y).step_by(CHUNK_SZ.y as usize - 1) {
                for z in 1..CHUNK_SZ.z - 1 {
                    gen_block_fn(x, y, z);
                }
            }
        }

        // z faces

        for x in 0..CHUNK_SZ.x {
            for y in 0..CHUNK_SZ.y {
                for z in (0..CHUNK_SZ.z).step_by(CHUNK_SZ.z as usize - 1) {
                    gen_block_fn(x, y, z);
                }
            }
        }

        // Can we make broad assumptions about the homogenity of the chunk?
        match cblock {
            (true, Some(block)) => return Chunk::Homo(
                HomogeneousData::filled(CHUNK_SZ, block)
            ),
            _ => {},
        }

        // Fill in everything else
        for x in 1..CHUNK_SZ.x - 1 {
            for y in 1..CHUNK_SZ.y - 1 {
                for z in 1..CHUNK_SZ.z - 1 {
                    let pos = offs.map(|e| e as i64) * CHUNK_SZ.map(|e| e as i64) + Vec3::new(x, y, z).map(|e| e as i64);

                    chunk_data.set_at(
                        Vec3::new(x, y, z),
                        GENERATOR.sample(pos).block,
                    );
                }
            }
        }

        Chunk::Hetero(chunk_data)
    }
}
