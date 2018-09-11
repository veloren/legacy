use vek::*;

use Block;
use Volume;
use Voxel;

use std::{any::Any, u8};

#[derive(Clone)]
pub struct ChunkFile {
    //per x and y coord store the z coord rle
    size: Vec3<i64>,
    offset: Vec3<i64>,
    file: &'static str,
}

impl Volume for ChunkFile {
    type VoxelType = Block;

    fn fill(&mut self, block: Block) {
        panic!("FEATURE NOT IMPLEMENTED YET: Cannot work on File Chunk");
    }

    fn size(&self) -> Vec3<i64> { self.size }

    fn offset(&self) -> Vec3<i64> { self.offset }

    fn ori(&self) -> Vec3<f32> { Vec3::new(0.0, 0.0, 0.0) }

    fn scale(&self) -> Vec3<f32> { Vec3::new(1.0, 1.0, 1.0) }

    fn set_size(&mut self, size: Vec3<i64>) {
        panic!("FEATURE NOT IMPLEMENTED YET: Cannot set size on File Chunk");
    }

    fn set_offset(&mut self, offset: Vec3<i64>) {
        panic!("FEATURE NOT IMPLEMENTED YET: Cannot set offset on File Chunk");
    }

    fn at(&self, pos: Vec3<i64>) -> Option<Block> {
        panic!("FEATURE NOT IMPLEMENTED YET: Cannot work on File Chunk");
    }

    fn set(&mut self, pos: Vec3<i64>, vt: Block) {
        panic!("FEATURE NOT IMPLEMENTED YET: Cannot work on File Chunk");
    }

    fn as_any(&mut self) -> &mut Any { self }
}

impl ChunkFile {
    pub fn new() -> Self {
        ChunkFile {
            size: Vec3::from((0, 0, 0)),
            offset: Vec3::from((0, 0, 0)),
            file: "",
        }
    }
}
