#![feature(nll, fn_traits, associated_type_defaults, self_struct_ctor)]

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

// Standard
use std::{mem, hash::Hash};

// Library
use vek::*;

// Project
use common::terrain::chunk::Chunk;

// Local
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

        let mut voxels = Vec::new();

        for x in 0..CHUNK_SZ.0 {
            for y in 0..CHUNK_SZ.1 {
                for z in 0..CHUNK_SZ.2 {
                    let pos = offs.map(|e| e as i64) * chunk_sz + Vec3::new(x, y, z).map(|e| e as i64);

                    voxels.push(GENERATOR.sample(pos).block);
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
