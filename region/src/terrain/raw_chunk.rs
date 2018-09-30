// Standard
use std::mem;

// Library
use vek::*;

// Local
use super::{block::Block, ConstructVolume, ReadVolume, Volume, Voxel, WriteVolume};

// What's going on here?
// ---------------------
// Below is a relatively well optimised chunk structure that is designed to use significantly
// less memory when containing a homogeneous block structure (i.e: every block within it is
// identical). We do this by using the ChunkData enum below, that has both a single-block variant
// for homogeneous structures and a Vec-backed multi-block variant for heterogeneous structures.
// The logic for manipulating this structure is obvious slightly more complex than a flat array.
// As a result, I've taken care to comment the code below with enough clarity such that you may
// (hopefully) understand what's going on.

#[derive(Clone, Debug)]
enum ChunkData {
    Homogeneous(Block),
    Heterogeneous(Vec<Block>),
}

#[derive(Clone, Debug)]
pub struct RawChunk {
    sz: Vec3<u64>,
    data: ChunkData,
}

impl RawChunk {
    // Return the given offet if it is within the chunk bounds or None otherwise
    fn validate_offset(&self, off: Vec3<u64>) -> Option<Vec3<u64>> {
        if off.x >= 0 && off.x < self.sz.x && off.y >= 0 && off.y < self.sz.y && off.z >= 0 && off.z < self.sz.z {
            Some(off)
        } else {
            None
        }
    }

    // Returns the vector index that corresponds with the given offet, None otherwise
    fn calculate_index(&self, off: Vec3<u64>) -> Option<usize> {
        self.validate_offset(off)
            .map(|off| (off.x * self.sz.y * self.sz.z + off.y * self.sz.z + off.z) as usize)
    }
}

impl Volume for RawChunk {
    type Voxel = Block;

    fn get_size(&self) -> Vec3<u64> { self.sz }

    fn maintain(&mut self) {
        match &mut self.data {
            ChunkData::Homogeneous(_) => {},
            ChunkData::Heterogeneous(blocks) => {
                // If the volume is 'considered' heterogeneous, there is a possibility that the
                // volume may have been altered in such a way as to make it homogeneous. To check
                // whether this is the case, we test whether all blocks are equivalent. If this is
                // the case, we can replace the internal data with the homogeneous variant to
                // improve both memory and computation efficiency.
                if blocks.windows(2).all(|vs| vs[0] == vs[1]) {
                    self.data = ChunkData::Homogeneous(*blocks.iter().next().unwrap_or(&Block::empty()));
                }
            },
        }
    }
}

impl ReadVolume for RawChunk {
    fn get_at(&self, off: Vec3<u64>) -> Option<Self::Voxel> {
        match &self.data {
            ChunkData::Homogeneous(block) => self.validate_offset(off).map(|_| *block),
            ChunkData::Heterogeneous(blocks) => self
                .calculate_index(off)
                .and_then(|idx| blocks.get(idx).map(|block| *block)),
        }
    }

    fn is_homo(&self) -> bool {
        match self.data {
            ChunkData::Homogeneous(_) => true,
            ChunkData::Heterogeneous(_) => false,
        }
    }
}

impl WriteVolume for RawChunk {
    fn replace_at(&mut self, off: Vec3<u64>, mut vox: Self::Voxel) -> Option<Self::Voxel> {
        match &mut self.data {
            ChunkData::Homogeneous(block) => {
                let block = *block;
                // If the data is homogeneous, and already contains the same block we're trying to
                // set, we do nothing. If it's not the same then we have to upgrade this chunk to
                // a heterogeneous chunk (i.e: it uses a shit-tonne of RAM).
                if block == vox {
                    Some(block)
                } else {
                    // Upgrade the chunk and replace the block if the offet is within the chunk
                    // bounds
                    self.calculate_index(off).map(|idx| {
                        let mut blocks = vec![block; self.sz.product() as usize];
                        blocks.get_mut(idx).map(|blk| mem::swap(blk, &mut vox));
                        self.data = ChunkData::Heterogeneous(blocks);
                        vox
                    })
                }
            },
            ChunkData::Heterogeneous(blocks) => {
                // TODO: This little bit is kind of horrid. We do this swapping thing to keep the
                // borrow checker happy. If you can find a way to neaten this, please do!
                let mut tmp = vec![];
                mem::swap(blocks, &mut tmp);
                // Replace the block if the offet is within the chunk bounds
                self.calculate_index(off)
                    .map(|idx| tmp.get_mut(idx).map(|blk| mem::swap(blk, &mut vox)));
                self.data = ChunkData::Heterogeneous(tmp);
                Some(vox)
            },
        }
    }

    fn fill(&mut self, vox: Self::Voxel) { self.data = ChunkData::Homogeneous(vox); }
}

impl ConstructVolume for RawChunk {
    fn filled(sz: Vec3<u64>, vox: Self::Voxel) -> Self {
        Self {
            sz,
            data: ChunkData::Homogeneous(vox),
        }
    }
}
