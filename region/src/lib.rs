#![feature(nll, euclidean_division)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate enum_map;
extern crate bincode;
extern crate nalgebra;
extern crate noise;
extern crate rand;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate coord;
extern crate threadpool;
#[macro_use]
extern crate lazy_static;
extern crate common;
extern crate parking_lot;

pub mod chunk;
mod collision;
mod container;
mod entity;
pub mod figure;
pub mod physics;
#[cfg(test)]
mod tests;
mod vol_gen;
mod vol_mgr;
mod vol_pers;

// Reexports
pub use container::{Container, VolContainer};
pub use entity::Entity;
pub use vol_gen::{FnGenFunc, FnPayloadFunc, VolGen};
pub use vol_mgr::{VolMgr, VolState};
pub use vol_pers::VolPers;

// Project
use coord::prelude::*;
use std::{any::Any, cmp::Eq, fmt::Debug, hash::Hash};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum PersState {
    Raw,
    Rle,
    File,
    //Network,
}

pub trait Key: Copy + Eq + Hash + Debug + 'static {
    fn print(&self) -> String;
}

pub trait VolConverter<C: VolContainer> {
    fn convert<K: Key>(key: &K, container: &mut C, state: PersState);
}

pub trait Voxel: Copy + Clone + Any {
    type Material: Copy + Clone;
    fn empty() -> Self;
    fn new(mat: Self::Material) -> Self;
    fn is_solid(&self) -> bool;
    fn material(&self) -> Self::Material;
}

pub trait Volume: Send + Sync + Any {
    type VoxelType: Voxel;

    fn fill(&mut self, block: Self::VoxelType);

    // number of Voxel in x, and z direction
    fn size(&self) -> Vec3<i64>;
    // offset of first Voxel in a hypothetical bigger Volume, e.g. offset = (50,0,0) means there is exactly space for another volume with offset (0,0,0) and size 50.
    fn offset(&self) -> Vec3<i64>;
    // orientation on the 3 axis in rad
    fn ori(&self) -> Vec3<f32>;
    // scale is applied to size and offset
    fn scale(&self) -> Vec3<f32>;

    // returns the size of the contained data
    //fn byte_size(&self) -> u64;
    // returns the size of the contained data hold in memory
    //TODO: sizeof?
    //fn memory_size(&self) -> u64;

    fn as_any_mut(&mut self) -> &mut Any;
    fn as_any(&self) -> &Any;

    fn set_size(&mut self, size: Vec3<i64>);
    fn set_offset(&mut self, offset: Vec3<i64>);

    fn at(&self, pos: Vec3<i64>) -> Option<Self::VoxelType>;
    fn set(&mut self, pos: Vec3<i64>, vt: Self::VoxelType);
}

/*

mod block;
mod chunk_container;
mod chunk_conv;
mod chunk_file;
mod chunk_rle;
mod chunk;
mod container;
mod vol_mgr;
mod vol_pers;

use std::{
    any::Any,
    cmp::Eq,
    fmt::Debug,
    hash::Hash,
};

use coord::prelude::*;

// Reexports
pub use self::{
    block::{BlockMaterial, Block},
    chunk_container::{ChunkContainer},
    chunk_conv::{ChunkConverter},
    chunk_file::{ChunkFile},
    chunk_rle::{ChunkRle, BlockRle, BLOCK_RLE_MAX_CNT},
    chunk::{Chunk},
    container::{VolContainer, Container},
    vol_mgr::{VolState, FnGenFunc, FnPayloadFunc, VolGen, VolMgr, VolMgrIter},
    vol_pers::{VolPers},
};

*/
