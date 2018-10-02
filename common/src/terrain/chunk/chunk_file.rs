use vek::*;

use terrain::{chunk::Block, Volume};

use std::any::Any;

#[derive(Clone)]
pub struct ChunkFile {
    //per x and y coord store the z coord rle
    size: Vec3<i64>,
    offset: Vec3<i64>,
    file: String,
}

impl Volume for ChunkFile {
    type VoxelType = Block;

    fn fill(&mut self, _block: Block) {
        panic!("FEATURE NOT IMPLEMENTED YET: Cannot work on File Chunk");
    }

    fn size(&self) -> Vec3<i64> { self.size }

    fn offset(&self) -> Vec3<i64> { self.offset }

    fn ori(&self) -> Vec3<f32> { Vec3::new(0.0, 0.0, 0.0) }

    fn scale(&self) -> Vec3<f32> { Vec3::new(1.0, 1.0, 1.0) }

    fn set_size(&mut self, _size: Vec3<i64>) {
        panic!("FEATURE NOT IMPLEMENTED YET: Cannot set size on File Chunk");
    }

    fn set_offset(&mut self, offset: Vec3<i64>) { self.offset = offset; }

    fn at(&self, _pos: Vec3<i64>) -> Option<Block> {
        panic!("FEATURE NOT IMPLEMENTED YET: Cannot work on File Chunk");
    }

    fn set(&mut self, _pos: Vec3<i64>, _vt: Block) {
        panic!("FEATURE NOT IMPLEMENTED YET: Cannot work on File Chunk");
    }

    fn as_any_mut(&mut self) -> &mut Any { self }

    fn as_any(&self) -> &Any { self }
}

impl ChunkFile {
    pub fn new(size: Vec3<i64>) -> Self {
        ChunkFile {
            size,
            offset: Vec3::from((0, 0, 0)),
            file: "".to_string(),
        }
    }

    pub fn file(&mut self) -> &String { return &self.file; }

    pub fn file_mut(&mut self) -> &mut String { return &mut self.file; }
}
