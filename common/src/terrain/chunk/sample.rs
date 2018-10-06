// Standard
use std::{collections::{HashMap, hash_map}, sync::Arc};

use terrain;
use terrain::{Volume, Voxel, ReadVolume, VoxelAbsType, VoxelRelType, VoxelRelVec, VoxelAbsVec, VolumeIdxVec};
use terrain::chunk::{ChunkContainer, Block, Chunk};
use parking_lot::{RwLock, RwLockReadGuard};

/// a ChunkSample is no real chunk, but like a pointer to real blocks
/// The ChunkSample can access blocks over multiple chunks like it's one coherent structure
/// It should be used when accessing blocks over chunk boundries because it can optimize the locking and read access

pub struct ChunkSample<'a, P> {
    dummy: Option<P>,
    // blocks in chunk, e.g. (16,16,16)
    vol_size: VoxelRelVec,
    // chunk_from + chunk_length-1 = chunk_to, just for optimisations store all 3!
    chunk_from: VolumeIdxVec,
    chunk_length: VolumeIdxVec,
    chunk_to: VolumeIdxVec,
    // block_from* + block_length-1 = block_from*, just for optimisations store all 3!
    block_from_abs: VoxelAbsVec,
    block_from_rel: VoxelRelVec,
    block_length: VoxelAbsVec,
    block_to_abs: VoxelAbsVec,
    block_to_rel: VoxelRelVec,

    // Store the absolute Chunk Index and a combination of the Chunk Container (only used for lifetimes) and the correct lock which is accessed
    // Dont try to lock on the ChunkContainer inside coding except for creation phase!
    //TODO: we currently store the Chunk here, so we might have different chunk formats inside a VolSample, evaluate if there are problems or if this is fine
    //TODO: optimize the Arc<Option away
    map: HashMap<VolumeIdxVec, Arc<RwLockReadGuard<'a, Chunk>>>,
    //map: HashMap<VolumeIdxVec, (Option<Arc<ChunkContainer<P>>>, Option<RwLockReadGuard<'a, Chunk>>)>,
}

pub struct ChunkSampleIter<'a, P> {
    dummy: Option<P>,
    owner: &'a ChunkSample<'a, P>,
    chunkiter: hash_map::Iter<'a, VolumeIdxVec, Arc<RwLockReadGuard<'a, Chunk>>>,
    //chunkiter: hash_map::Iter<'a, VolumeIdxVec, (Option<Arc<ChunkContainer<P>>>, Option<RwLockReadGuard<'a, Chunk>>)>,
    chunkiteritem: Option<(&'a VolumeIdxVec, &'a Arc<RwLockReadGuard<'a, Chunk>>)>,
    //chunkiteritem: Option<(&'a VolumeIdxVec, &'a (Option<Arc<ChunkContainer<P>>>, Option<RwLockReadGuard<'a, Chunk>>))>,
    block_rel: VoxelRelVec,
}

impl<'a, P> ChunkSampleIter<'a, P> {
    pub fn new(sample: &'a ChunkSample<'a, P>) -> Self {
        let s = sample;
        let i = s.map.iter();
        let mut csi = ChunkSampleIter{
            dummy: None,
            owner: &s,
            chunkiter: i,
            chunkiteritem: None,
            block_rel: s.block_from_rel,
        };
        csi.chunkiteritem = csi.chunkiter.next();
        return csi;
    }
}

impl<'a, P> Iterator for ChunkSampleIter<'a, P> {
    type Item = (VoxelAbsVec, Block);

    fn next(&mut self) -> Option<Self::Item> {
        if self.chunkiteritem.is_none() {
            self.chunkiteritem = self.chunkiter.next();
            self.block_rel = VoxelRelVec::new(0,0,0);
        }
        if let Some((key, item)) = self.chunkiteritem {
            let rel = self.block_rel;
            let abs = key.map(|e| e as VoxelAbsType) + rel.map(|e| e as VoxelAbsType);
            let b = ChunkSample::<'a, P>::access(&item, rel);
            self.block_rel.z += 1;
            if self.block_rel.z == self.owner.vol_size.z {
                self.block_rel.z = 0;
                self.block_rel.y += 1;
                if self.block_rel.y == self.owner.vol_size.y {
                    self.block_rel.y = 0;
                    self.block_rel.x += 1;
                    if self.block_rel.x == self.owner.vol_size.x {
                        self.block_rel.x = 0;
                        self.chunkiteritem = None;
                    }
                }
            }
            return Some((abs, b));
        }
        None
    }
}

impl<'a, P> ChunkSample<'a, P> {
    pub fn new(
        vol_size: VoxelRelVec,
        block_from_abs: VoxelAbsVec,
        block_to_abs: VoxelAbsVec,
        map: HashMap<VolumeIdxVec, Arc<RwLockReadGuard<'a, Chunk>>>
        //map: HashMap<VolumeIdxVec, (Option<Arc<ChunkContainer<P>>>, Option<RwLockReadGuard<'a, Chunk>>)>,
    ) -> Self {
        let block_from_rel = terrain::voxabs_to_voxrel(block_from_abs, vol_size);
        let block_to_rel = terrain::voxabs_to_voxrel(block_to_abs, vol_size);
        let block_length = block_to_abs - block_from_abs + VoxelAbsVec::new(1,1,1);
        let chunk_from = terrain::voxabs_to_volidx(block_from_abs, vol_size);
        let chunk_to = terrain::voxabs_to_volidx(block_to_abs, vol_size);
        let chunk_length = chunk_to - chunk_from + VolumeIdxVec::new(1,1,1);
        ChunkSample{
            dummy: None,
            vol_size,
            chunk_from,
            chunk_length,
            chunk_to,
            block_from_abs,
            block_from_rel,
            block_length,
            block_to_abs,
            block_to_rel,
            map: map,
        }
    }

    pub fn iter(&'a self) -> ChunkSampleIter<'a, P> {
        ChunkSampleIter::new(self)
    }

    fn access(lock: &RwLockReadGuard<'a, Chunk>, off: VoxelRelVec) -> Block {
        match **lock {
            Chunk::Homogeneous{ ref homo } => {
                if let Some(homo) = homo {
                    return homo.at_unsafe(off);
                } else {
                    panic!("No Homo Chunk available");
                }
            },
            Chunk::Heterogeneous{ ref hetero, ref rle} => {
                // Hetero first, maybe we should make a trait for this ?
                if let Some(hetero) = hetero {
                    return hetero.at_unsafe(off);
                } else {
                    if let Some(rle) = rle {
                        return rle.at_unsafe(off);
                    } else {
                        panic!("Neither Hetero or Rle Chunk available");
                    }
                }
            },
        };
    }

    pub fn at_abs(&self, off: VoxelAbsVec) -> Option<Block> {
        let size = self.size();
        let chunkidx = terrain::voxabs_to_volidx(off, size);
        let blockrel = terrain::voxabs_to_voxrel(off, size);
        let _ = self.map.get(&chunkidx).map(|lock| {
            return Some(ChunkSample::<'a, P>::access(&lock, blockrel));
        });
        None
    }

    pub fn at_abs_unsafe(&self, off: VoxelAbsVec) -> Block {
        let size = self.size();
        let chunkidx = terrain::voxabs_to_volidx(off, size);
        let blockrel = terrain::voxabs_to_voxrel(off, size);
        let _ = self.map.get(&chunkidx).map(|lock| {
            return ChunkSample::<'a, P>::access(&lock, blockrel);
        });
        panic!("off not inside VolSample: {}, chunkidx: {}", off, chunkidx);
    }

    pub fn size_chunks(&self) -> VolumeIdxVec {
        self.chunk_length
    }

    pub fn size_blocks(&self) -> VoxelAbsVec {
        self.block_length
    }
}

impl<'a, P> Volume for ChunkSample<'a, P> {
    type VoxelType = Block;

    fn size(&self) -> VoxelRelVec {
        //TODO: This conversion is potentialy DANGEROUS! because we say mix implementaton with interface here, thing about it carefully.Will make problems when sampling over 4096 chunks for now!
        self.block_length.map(|e| e as VoxelRelType)
    }
}

impl<'a, P> ReadVolume for ChunkSample<'a, P> {
    fn at_unsafe(&self, pos: VoxelRelVec) -> Block {
        let abs = self.block_from_abs + pos.map(|e| e as VoxelAbsType);
        self.at_abs_unsafe(abs)
    }
}
