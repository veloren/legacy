// Standard
use std::u8;

// Library
use vek::*;

// Local
use terrain::{chunk::Block, ConstructVolume, PhysicalVolume, ReadVolume, Volume, VoxRel, Voxel};

//TODO: optimizations:
// currently even empty blocks generate a BlockRle, one could say that if the 3rd vector is empty that all blocks are empty

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct BlockRle {
    pub block: Block,
    pub num_minus_one: u8, // num_minus_one = 0 --> num is 1 and 255->256
}
pub const BLOCK_RLE_MAX_NUM: VoxRel = u8::MAX as VoxRel + 1;

impl BlockRle {
    pub fn new(block: Block, num_minus_one: u8) -> Self { BlockRle { block, num_minus_one } }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RleData {
    //per x and y coord store the z coord rle
    size: Vec3<VoxRel>,
    voxels: Vec<Vec<BlockRle>>,
}

impl RleData {
    pub(crate) fn voxels_mut(&mut self) -> &mut Vec<Vec<BlockRle>> { &mut self.voxels }

    pub fn voxels_mut_internal(&mut self) -> &mut Vec<Vec<BlockRle>> { &mut self.voxels }
}

impl Volume for RleData {
    type VoxelType = Block;

    fn size(&self) -> Vec3<VoxRel> { self.size }
}

impl ReadVolume for RleData {
    fn at_unchecked(&self, pos: Vec3<VoxRel>) -> Block {
        let col = &self.voxels[pos.x as usize * self.size.y as usize + pos.y as usize];
        let mut oldz: VoxRel = 0;
        for brle in col {
            let z: VoxRel = oldz + brle.num_minus_one as VoxRel + 1;
            if pos.z >= oldz && pos.z < z {
                return brle.block;
            }
            oldz = z;
        }
        Block::empty()
    }
}

impl ConstructVolume for RleData {
    fn filled(size: Vec3<VoxRel>, vox: Self::VoxelType) -> RleData {
        let mut rle = RleData {
            size,
            voxels: vec![Vec::new(); size.x as usize * size.y as usize],
        };
        let high = ((size.z as VoxRel) / (BLOCK_RLE_MAX_NUM)) as usize;
        let lastsize = size.z % (BLOCK_RLE_MAX_NUM);

        for xy in rle.voxels.iter_mut() {
            xy.resize(high + 1, BlockRle::new(vox, 0));
            xy.iter_mut().map(|e| e.num_minus_one = (BLOCK_RLE_MAX_NUM - 1) as u8);
            xy.last_mut().unwrap().num_minus_one = lastsize as u8;
        }

        rle
    }

    fn empty(size: Vec3<VoxRel>) -> RleData { Self::filled(size, Block::empty()) }
}

impl PhysicalVolume for RleData {}
