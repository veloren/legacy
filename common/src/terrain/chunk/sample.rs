// Standard
use std::{
    collections::{hash_map, HashMap},
    sync::Arc,
};

// Library
use parking_lot::RwLockReadGuard;
use vek::*;

// Local
use crate::terrain::{
    self,
    chunk::{Block, Chunk},
    ReadVolume, VolOffs, Volume, VoxAbs, VoxRel,
};

/// a ChunkSample is no real chunk, but like a pointer to real blocks
/// The ChunkSample can access blocks over multiple chunks like it's one coherent structure
/// It should be used when accessing blocks over chunk boundries because it can optimize the locking and read access

pub struct ChunkSample<'a> {
    vol_size: Vec3<VoxRel>, // blocks in chunk, e.g. (16,16,16)
    block_from_abs: Vec3<VoxAbs>,
    block_from_rel: Vec3<VoxRel>,
    block_length: Vec3<VoxAbs>,
    block_to_abs: Vec3<VoxAbs>,

    // Store the absolute Chunk Index and the correct lock which is used inside ChunkSample
    map: HashMap<Vec3<VolOffs>, Arc<RwLockReadGuard<'a, Chunk>>>,
}

pub struct ChunkSampleIter<'a> {
    owner: &'a ChunkSample<'a>,
    chunkiter: hash_map::Iter<'a, Vec3<VolOffs>, Arc<RwLockReadGuard<'a, Chunk>>>,
    chunkiteritem: Option<(&'a Vec3<VolOffs>, &'a Arc<RwLockReadGuard<'a, Chunk>>)>,
    block_rel: Vec3<VoxRel>,
}

impl<'a> Iterator for ChunkSampleIter<'a> {
    type Item = (Vec3<VoxAbs>, Block);

    fn next(&mut self) -> Option<Self::Item> {
        if self.chunkiteritem.is_none() {
            self.chunkiteritem = self.chunkiter.next();
            self.block_rel = Vec3::new(0, 0, 0);
        }
        if let Some((key, item)) = self.chunkiteritem {
            let abs = terrain::voloffs_to_voxabs(*key, self.owner.vol_size) + self.block_rel.map(|e| e as VoxAbs);
            if abs.x < self.owner.block_from_abs.x {
                self.block_rel.x = self.owner.block_from_rel.x;
            }
            if abs.y < self.owner.block_from_abs.y {
                self.block_rel.y = self.owner.block_from_rel.y;
            }
            if abs.z < self.owner.block_from_abs.z {
                self.block_rel.z = self.owner.block_from_rel.z;
            }

            let b = ChunkSample::<'a>::access(&item, self.block_rel);
            self.block_rel.x += 1;
            if self.block_rel.x == self.owner.vol_size.x || abs.x > self.owner.block_to_abs.x {
                self.block_rel.x = 0;
                self.block_rel.y += 1;
                if self.block_rel.y == self.owner.vol_size.y || abs.y > self.owner.block_to_abs.y {
                    self.block_rel.y = 0;
                    self.block_rel.z += 1;
                    if self.block_rel.z == self.owner.vol_size.z || abs.z > self.owner.block_to_abs.z {
                        self.block_rel.z = 0;
                        self.chunkiteritem = None;
                    }
                }
            }
            return Some((abs, b));
        }
        None
    }
}

impl<'a> ChunkSample<'a> {
    pub(crate) fn new_internal(
        vol_size: Vec3<VoxRel>,
        block_from_abs: Vec3<VoxAbs>,
        block_to_abs: Vec3<VoxAbs>,
        map: HashMap<Vec3<VolOffs>, Arc<RwLockReadGuard<'a, Chunk>>>,
    ) -> Self {
        ChunkSample {
            vol_size,
            block_from_abs,
            block_from_rel: terrain::voxabs_to_voxrel(block_from_abs, vol_size),
            block_length: block_to_abs - block_from_abs + Vec3::new(1, 1, 1),
            block_to_abs,
            map: map,
        }
    }

    pub fn iter(&'a self) -> ChunkSampleIter<'a> {
        ChunkSampleIter {
            owner: &self,
            chunkiter: self.map.iter(),
            chunkiteritem: None,
            block_rel: Vec3::new(0, 0, 0),
        }
    }

    fn access(lock: &RwLockReadGuard<'a, Chunk>, off: Vec3<VoxRel>) -> Block {
        match **lock {
            Chunk::Homo(ref homo) => homo.at_unchecked(off),
            Chunk::Hetero(ref hetero) => hetero.at_unchecked(off),
            Chunk::Rle(ref rle) => rle.at_unchecked(off),
            Chunk::HeteroAndRle(ref hetero, _) => hetero.at_unchecked(off),
        }
    }

    pub fn at_abs(&self, off: Vec3<VoxAbs>) -> Option<Block> {
        let size = self.size();
        let chunkidx = terrain::voxabs_to_voloffs(off, size);
        let blockrel = terrain::voxabs_to_voxrel(off, size);
        let _ = self.map.get(&chunkidx).map(|lock| {
            return Some(ChunkSample::<'a>::access(&lock, blockrel));
        });
        None
    }

    pub fn at_abs_unchecked(&self, off: Vec3<VoxAbs>) -> Block {
        let size = self.size();
        let chunkidx = terrain::voxabs_to_voloffs(off, size);
        let blockrel = terrain::voxabs_to_voxrel(off, size);
        let _ = self.map.get(&chunkidx).map(|lock| {
            return ChunkSample::<'a>::access(&lock, blockrel);
        });
        panic!("off not inside VolSample: {}, chunkidx: {}", off, chunkidx);
    }

    pub fn size_blocks(&self) -> Vec3<VoxAbs> { self.block_length }
}

impl<'a> Volume for ChunkSample<'a> {
    type VoxelType = Block;

    fn size(&self) -> Vec3<VoxRel> {
        //TODO: This conversion is potentialy DANGEROUS! because we say mix implementaton with interface here, thing about it carefully.Will make problems when sampling over 4096 chunks for now!
        self.block_length.map(|e| e as VoxRel)
    }
}

impl<'a> ReadVolume for ChunkSample<'a> {
    fn at_unchecked(&self, pos: Vec3<VoxRel>) -> Block {
        let abs = self.block_from_abs + pos.map(|e| e as VoxAbs);
        self.at_abs_unchecked(abs)
    }
}
