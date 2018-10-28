// Standard
use std::{u8};

// Library
use vek::*;

// Local
use terrain::{
    chunk::{Block},
    Volume, ReadVolume, ConstructVolume, PhysicalVolume, Voxel, VoxelRelVec,
};

//TODO: optimizations:
// currently even empty blocks generate a BlockRle, one could say that if the 3rd vector is empty that all blocks are empty

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct BlockRle {
    pub block: Block,
    pub num_minus_one: u8, // num_minus_one = 0 --> num is 1 and 255->256
}
pub const BLOCK_RLE_MAX_NUM: u16 = u8::MAX as u16 + 1;

impl BlockRle {
    pub fn new(block: Block, num_minus_one: u8) -> Self { BlockRle { block, num_minus_one } }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RleData {
    //per x and y coord store the z coord rle
    size: VoxelRelVec,
    voxels: Vec<Vec<BlockRle>>,
}

impl RleData {
    fn new() -> Self {
        RleData {
            size: Vec3::from((0, 0, 0)),
            voxels: vec![],
        }
    }

    pub(crate) fn voxels_mut(&mut self) -> &mut Vec<Vec<BlockRle>> {
        &mut self.voxels
    }

    pub fn voxels_mut_internal(&mut self) -> &mut Vec<Vec<BlockRle>> {
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
        let mut rle = RleData{
            size,
            voxels: vec![Vec::new(); size.x as usize * size.y as usize],
        };
        let high = ((size.z as u16) / (BLOCK_RLE_MAX_NUM)) as usize;
        let lastsize = size.z % (BLOCK_RLE_MAX_NUM);

        for xy in rle.voxels.iter_mut() {
            xy.resize(high+1, BlockRle::new(vox, 0));
            xy.iter_mut().map(|e| e.num_minus_one = (BLOCK_RLE_MAX_NUM-1) as u8);
            xy.last_mut().unwrap().num_minus_one = lastsize as u8;
        };

        rle
    }

    fn empty(size: VoxelRelVec) -> RleData {
        Self::filled(size, Block::empty())
    }
}

impl PhysicalVolume for RleData {
}
