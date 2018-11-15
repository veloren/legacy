mod block;
mod cluster;
mod container;
mod hetero;
mod homo;
mod rle;
mod sample;
#[cfg(test)]
mod tests;

// Library
use vek::*;

// Reexports
pub use self::{
    block::{Block, BlockMat},
    cluster::Chunk,
    container::ChunkContainer,
    hetero::HeterogeneousData,
    homo::HomogeneousData,
    rle::{BlockRle, RleData, BLOCK_RLE_MAX_NUM},
    sample::ChunkSample,
};

// Local
use terrain::VoxRel;

pub const CHUNK_SIZE: Vec3<VoxRel> = Vec3 { x: 32, y: 32, z: 32 };
