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
    },
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

const CHUNK_SZ: (u32, u32, u32) = (64, 64, 64);

lazy_static! {
    static ref GENERATOR: TopologyGen = TopologyGen::new();
}

pub struct World;

impl World {
    pub fn gen_chunk(offs: Vec3<i32>) -> Chunk {
        let chunk_sz = Vec3::from(CHUNK_SZ).map(|e: u32| e as i64);

        let mut chunk = Chunk::new(
            chunk_sz,
            offs.map(|e| e as i64),
            vec![Block::AIR; chunk_sz.product() as usize],
        );

        if offs.z < 0 || offs.z > 8 {
            return chunk;
        }

        // is_homogeneous, block type
        let mut cblock = (true, None);

        let mut gen_block_fn = |x, y, z| {
            let pos = offs.map(|e| e as i64) * chunk_sz + Vec3::new(x, y, z);

            let block = GENERATOR.sample(pos).block;

            match cblock {
                (true, None) => cblock.1 = Some(block),
                (true, Some(b)) if b == block => {},
                (true, Some(b)) => cblock = (false, None),
                _ => {},
            }

            chunk.set(
                Vec3::new(x, y, z),
                block,
            );
        };

        // x faces

        for x in (0..chunk_sz.x).step_by(chunk_sz.x as usize - 1) {
            for y in 1..chunk_sz.y - 1 {
                for z in 1..chunk_sz.z - 1 {
                    gen_block_fn(x, y, z);
                }
            }
        }

        // y faces

        for x in 0..chunk_sz.x {
            for y in (0..chunk_sz.y).step_by(chunk_sz.y as usize - 1) {
                for z in 1..chunk_sz.z - 1 {
                    gen_block_fn(x, y, z);
                }
            }
        }

        // z faces

        for x in 0..chunk_sz.x {
            for y in 0..chunk_sz.y {
                for z in (0..chunk_sz.z).step_by(chunk_sz.z as usize - 1) {
                    gen_block_fn(x, y, z);
                }
            }
        }

        // Can we make broad assumptions about the homogenity of the chunk?
        match cblock {
            (true, Some(block)) => return Chunk::new(
                chunk_sz,
                offs.map(|e| e as i64),
                vec![block; chunk_sz.product() as usize],
            ),
            _ => {},
        }

        // Fill in everything else
        for x in 1..chunk_sz.x - 1 {
            for y in 1..chunk_sz.y - 1 {
                for z in 1..chunk_sz.z - 1 {
                    let pos = offs.map(|e| e as i64) * chunk_sz + Vec3::new(x, y, z);

                    chunk.set(
                        Vec3::new(x, y, z),
                        GENERATOR.sample(pos).block,
                    );
                }
            }
        }

        chunk
    }
}
