#![feature(
    nll,
    fn_traits,
    associated_type_defaults,
    self_struct_ctor,
    euclidean_division,
    integer_atomics
)]

mod blockgen;
mod cachegen;
mod overworldgen;
mod towngen;
mod util;

// Standard
use std::sync::atomic::{AtomicU32, Ordering};

// Library
use lazy_static::lazy_static;
use vek::*;

// Project
use common::terrain::{
    chunk::{Block, Chunk, HeterogeneousData, HomogeneousData, CHUNK_SIZE},
    ConstructVolume, ReadWriteVolume,
};

// Local
use crate::blockgen::BlockGen;

// Generator

pub trait Gen<S> {
    type In: Clone;
    type Out: Clone;

    fn sample<'a>(&'a self, i: Self::In, supplement: &'a S) -> Self::Out;
}

// Seed - used during worldgen initiation
static SEED: AtomicU32 = AtomicU32::new(0);
pub fn new_seed() -> u32 { SEED.fetch_add(1, Ordering::Relaxed) }

lazy_static! {
    static ref GENERATOR: BlockGen = BlockGen::new();
}

pub struct World;

impl World {
    pub fn gen_chunk(offs: Vec3<i32>) -> Chunk {
        // If the chunk is out of bounds, just generate air
        if offs.z < 0 || offs.z > 512 / CHUNK_SIZE.z as i32 {
            return Chunk::Homo(HomogeneousData::filled(CHUNK_SIZE, Block::AIR));
        }

        let mut chunk_data = HeterogeneousData::empty(CHUNK_SIZE);
        let generator = &GENERATOR; // Create a temporary for the generator here to avoid atomic operations for every block

        // is_homogeneous, block type
        let mut cblock = (true, None);

        let mut gen_block_fn = |x, y, z| {
            let pos = offs.map(|e| e as i64) * CHUNK_SIZE.map(|e| e as i64) + Vec3::new(x, y, z).map(|e| e as i64);

            let block = generator.sample(pos, &generator.get_invariant_z(Vec2::from(pos)));

            match cblock {
                (true, None) => cblock.1 = Some(block),
                (true, Some(b)) if b == block => {},
                (true, Some(_)) => cblock = (false, None),
                _ => {},
            }

            chunk_data.set_at(Vec3::new(x, y, z), block);
        };

        // x faces

        for x in (0..CHUNK_SIZE.x).step_by(CHUNK_SIZE.x as usize - 1) {
            for y in 1..CHUNK_SIZE.y - 1 {
                for z in 1..CHUNK_SIZE.z - 1 {
                    gen_block_fn(x, y, z);
                }
            }
        }

        // y faces

        for x in 0..CHUNK_SIZE.x {
            for y in (0..CHUNK_SIZE.y).step_by(CHUNK_SIZE.y as usize - 1) {
                for z in 1..CHUNK_SIZE.z - 1 {
                    gen_block_fn(x, y, z);
                }
            }
        }

        // z faces

        for x in 0..CHUNK_SIZE.x {
            for y in 0..CHUNK_SIZE.y {
                for z in (0..CHUNK_SIZE.z).step_by(CHUNK_SIZE.z as usize - 1) {
                    gen_block_fn(x, y, z);
                }
            }
        }

        // Can we make broad assumptions about the homogenity of the chunk?
        match cblock {
            (true, Some(block)) => return Chunk::Homo(HomogeneousData::filled(CHUNK_SIZE, block)),
            _ => {},
        }

        // Fill in everything else
        for x in 1..CHUNK_SIZE.x - 1 {
            for y in 1..CHUNK_SIZE.y - 1 {
                let pos2d = Vec2::from(offs.map(|e| e as i64)) * Vec2::from(CHUNK_SIZE.map(|e| e as i64))
                    + Vec2::new(x, y).map(|e| e as i64);
                let invariant_z = generator.get_invariant_z(pos2d);

                for z in 1..CHUNK_SIZE.z - 1 {
                    let pos =
                        offs.map(|e| e as i64) * CHUNK_SIZE.map(|e| e as i64) + Vec3::new(x, y, z).map(|e| e as i64);

                    chunk_data.set_at(Vec3::new(x, y, z), generator.sample(pos, &invariant_z));
                }
            }
        }

        Chunk::Hetero(chunk_data)
    }
}
