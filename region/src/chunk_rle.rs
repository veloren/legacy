use coord::prelude::*;

use Block;
use Volume;
use Voxel;

use std::{any::Any, u8};

//TODO: optimizations:
// currently even empty blocks generate a BlockRle, one could say that if the 3rd vector is empty that all blocks are empty
// then we could optimize the cnt variable, we could interpret 0 as 1, increasing our capacity from 255 to 256
// that means that no empty BlockRle would be allowed, but thats no problem

#[derive(Copy, Clone)]
pub struct BlockRle {
    pub block: Block,
    pub cnt: u8,
}
const MAX_CNT: u8 = u8::MAX;

impl BlockRle {
    fn new(block: Block, cnt: u8) -> Self { BlockRle { block, cnt } }
}

pub struct ChunkRle {
    //per x and y coord store the z coord rle
    size: Vec3<i64>,
    offset: Vec3<i64>,
    voxels: Vec<Vec<Vec<BlockRle>>>,
}

impl Volume for ChunkRle {
    type VoxelType = Block;

    fn fill(&mut self, block: Block) {
        let high = ((self.size.z as f32) / (MAX_CNT as f32)).ceil() as usize;
        let lastsize = self.size.z % (MAX_CNT as i64);
        for x in self.voxels.iter_mut() {
            for y in x.iter_mut() {
                y.resize(high, BlockRle::new(block, 0));
                y.iter_mut().map(|e| e.cnt = MAX_CNT);
                y.last_mut().unwrap().cnt = lastsize as u8;
            }
        }
    }

    fn size(&self) -> Vec3<i64> { self.size }

    fn offset(&self) -> Vec3<i64> { self.offset }

    fn ori(&self) -> Vec3<f32> { Vec3::new(0.0, 0.0, 0.0) }

    fn scale(&self) -> Vec3<f32> { Vec3::new(1.0, 1.0, 1.0) }

    fn set_size(&mut self, size: Vec3<i64>) {
        self.size = size;
        self.voxels.resize(size.x as usize, Vec::new());
        self.voxels.iter_mut().map(|e| e.resize(size.y as usize, Vec::new()));
        self.fill(Block::empty()); //TODO: only change new sizes, dont change everything
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
    pub fn voxels_mut(&mut self) -> &mut Vec<Vec<Vec<BlockRle>>> { &mut self.voxels }

    pub fn new() -> Self {
        ChunkRle {
            size: Vec3::from((0, 0, 0)),
            offset: Vec3::from((0, 0, 0)),
            voxels: Vec::new(),
        }
    }
}
