#![feature(nll, euclidean_division)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate enum_map;
extern crate nalgebra;
extern crate noise;
extern crate rand;
#[macro_use]
extern crate coord;
extern crate threadpool;
#[macro_use]
extern crate lazy_static;
extern crate common;

mod block;
mod cell;
mod chunk;
mod collision;
mod entity;
mod figure;
pub mod physics;
#[cfg(test)]
mod tests;
mod vol_mgr;

// Reexports
pub use block::{Block, BlockMaterial};
pub use cell::Cell;
pub use chunk::Chunk;
pub use entity::Entity;
pub use figure::Figure;
pub use vol_mgr::{FnGenFunc, FnPayloadFunc, VolGen, VolMgr, VolState};

use coord::prelude::*;

pub trait Voxel: Copy + Clone {
    type Material: Copy + Clone;
    fn empty() -> Self;
    fn new(mat: Self::Material) -> Self;
    fn is_solid(&self) -> bool;
    fn material(&self) -> Self::Material;
}

pub trait Volume: Send + Sync {
    type VoxelType: Voxel + Copy + Clone;

    fn new() -> Self;
    fn fill(&mut self, block: Self::VoxelType);

    // number of Voxel in x, and z direction
    fn size(&self) -> Vec3<i64>;
    // offset of first Voxel in a hypothetical bigger Volume, e.g. offset = (50,0,0) means there is exactly space for another volume with offset (0,0,0) and size 50.
    fn offset(&self) -> Vec3<i64>;
    // orientation on the 3 axis in rad
    fn ori(&self) -> Vec3<f32>;
    // scale is applied to size and offset
    fn scale(&self) -> Vec3<f32>;

    fn set_size(&mut self, size: Vec3<i64>);
    fn set_offset(&mut self, offset: Vec3<i64>);

    fn at(&self, pos: Vec3<i64>) -> Option<Self::VoxelType>;
    fn set(&mut self, pos: Vec3<i64>, vt: Self::VoxelType);
}
