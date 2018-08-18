use coord::prelude::*;

use Block;
use Volume;
use Voxel;

use std::{any::Any, u8};

//TODO: optimizations:
// currently even empty blocks generate a BlockRle, one could say that if the 3rd vector is empty that all blocks are empty
// then we could optimize the num variable, we could interpret 0 as 1, increasing our capacity from 255 to 256
// that means that no empty BlockRle would be allowed, but thats no problem

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct BlockRle {
    pub block: Block,
    pub num_minus_one: u8, // num_minus_one = 0 --> num is 1 and 255->256
}
pub const BLOCK_RLE_MAX_CNT: u8 = u8::MAX;

impl BlockRle {
    pub fn new(block: Block, num_minus_one: u8) -> Self { BlockRle { block, num_minus_one } }
}

#[derive(Clone, PartialEq, Debug)]
pub struct ChunkRle {
    //per x and y coord store the z coord rle
    size: Vec3<i64>,
    offset: Vec3<i64>,
    voxels: Vec<Vec<BlockRle>>,
    // Vec1= x und y kombinier, Vec2 z
}

impl Volume for ChunkRle {
    type VoxelType = Block;

    fn fill(&mut self, block: Block) {
        let high = ((self.size.z as f32) / (BLOCK_RLE_MAX_CNT as f32 + 1.0)).ceil() as usize;
        let lastsize = self.size.z % (BLOCK_RLE_MAX_CNT as i64 + 1);
        for xy in self.voxels.iter_mut() {
            xy.resize(high, BlockRle::new(block, 0));
            xy.iter_mut().map(|e| e.num_minus_one = BLOCK_RLE_MAX_CNT);
            xy.last_mut().unwrap().num_minus_one = lastsize as u8;
        }
    }

    fn size(&self) -> Vec3<i64> { self.size }

    fn offset(&self) -> Vec3<i64> { self.offset }

    fn ori(&self) -> Vec3<f32> { Vec3::new(0.0, 0.0, 0.0) }

    fn scale(&self) -> Vec3<f32> { Vec3::new(1.0, 1.0, 1.0) }

    fn set_size(&mut self, size: Vec3<i64>) {
        self.size = size;
        self.voxels.resize((size.x * size.y) as usize, Vec::new());
    }

    fn set_offset(&mut self, offset: Vec3<i64>) { self.offset = offset; }

    fn at(&self, pos: Vec3<i64>) -> Option<Block> {
        panic!("FEATURE NOT IMPLEMENTED YET: i dont feel like implement this now");
    }

    fn set(&mut self, pos: Vec3<i64>, vt: Block) {
        panic!("FEATURE NOT IMPLEMENTED YET: i dont feel like implement this now");
    }

    fn as_any(&mut self) -> &mut Any { self }
}

impl ChunkRle {
    pub fn voxels_mut(&mut self) -> &mut Vec<Vec<BlockRle>> { &mut self.voxels }

    pub fn new() -> Self {
        ChunkRle {
            size: Vec3::from((0, 0, 0)),
            offset: Vec3::from((0, 0, 0)),
            voxels: Vec::new(),
        }
    }

    fn pos_to_index(&self, x: i64, y: i64) -> usize { (x * self.size.y + y) as usize }
}
