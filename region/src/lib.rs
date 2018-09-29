#![feature(nll, euclidean_division, specialization)]
#![feature(option_replace)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate enum_map;
extern crate noise;
extern crate rand;
extern crate threadpool;
extern crate vek;
#[macro_use]
extern crate lazy_static;
extern crate common;
extern crate parking_lot;
extern crate serde;
extern crate specs;
#[macro_use]
extern crate serde_derive;

mod block;
mod cell;
mod chunk;
mod chunk_conv;
mod chunk_file;
mod chunk_rle;
mod collision;
mod entity;
mod figure;
pub mod physics;
#[cfg(test)]
mod tests;
mod vol_mgr;
mod vol_per;

pub mod ecs;
pub mod item;

// Reexports
pub use block::{Block, BlockMaterial};
pub use cell::{Cell, CellMaterial};
pub use chunk::Chunk;
pub use chunk_conv::{ChunkContainer, ChunkConverter};
pub use entity::Entity;
pub use figure::Figure;
pub use vol_mgr::{FnGenFunc, FnPayloadFunc, VolGen, VolMgr, VolState};
pub use vol_per::{Container, PersState, VolPers, VolumeConverter};

use std::any::Any;
use vek::*;

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

    fn as_any(&mut self) -> &mut Any;

    fn set_size(&mut self, size: Vec3<i64>);
    fn set_offset(&mut self, offset: Vec3<i64>);

    fn at(&self, pos: Vec3<i64>) -> Option<Self::VoxelType>;
    fn set(&mut self, pos: Vec3<i64>, vt: Self::VoxelType);
}
