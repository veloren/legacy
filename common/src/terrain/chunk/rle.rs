// Standard
use std::{u8};

// Library
use vek::*;

// Local
use terrain::{
    chunk::{Block},
    Volume, ReadVolume, ConstructVolume, PhysicallyVolume, Voxel, VoxelRelVec,
};

//TODO: optimizations:
// currently even empty blocks generate a BlockRle, one could say that if the 3rd vector is empty that all blocks are empty

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct BlockRle {
    pub block: Block,
    pub num_minus_one: u8, // num_minus_one = 0 --> num is 1 and 255->256
}
pub const BLOCK_RLE_MAX_CNT: u8 = u8::MAX;

impl BlockRle {
    pub fn new(block: Block, num_minus_one: u8) -> Self { BlockRle { block, num_minus_one } }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RleData {
    //per x and y coord store the z coord rle
    size: VoxelRelVec,
    voxels: Vec<Vec<BlockRle>>,
}

impl RleData {
    pub fn new() -> Self {
        RleData {
            size: Vec3::from((0, 0, 0)),
            voxels: Vec::new(),
        }
    }

    pub fn mut_size(&mut self) -> &mut VoxelRelVec {
        &mut self.size
    }

    pub fn mut_voxel(&mut self) -> &mut Vec<Vec<BlockRle>> {
        &mut self.voxels
    }
}

impl Volume for RleData {
    type VoxelType = Block;

    fn size(&self) -> VoxelRelVec { self.size }
}

impl ReadVolume for RleData {
    fn at_unsafe(&self, pos: VoxelRelVec) -> Block {
        panic!("FEATURE NOT IMPLEMENTED YET: i dont feel like implement this now");
    }
}

impl ConstructVolume for RleData {
    fn filled(size: VoxelRelVec, vox: Self::VoxelType) -> RleData {
        RleData{
            size,
            voxels: vec![Vec::new(); (size.x * size.y) as usize],
        }
    }

    fn empty(size: VoxelRelVec) -> RleData {
        Self::filled(size, Block::empty())
    }
}

impl PhysicallyVolume for RleData {
    fn scale(&self) -> Vec3<f32> {
        Vec3::new(1.0, 1.0, 1.0)
    }
}
