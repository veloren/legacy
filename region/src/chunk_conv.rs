use chunk::Chunk;
use chunk_file::ChunkFile;
use chunk_rle::{BlockRle, ChunkRle, BLOCK_RLE_MAX_CNT};
use coord::prelude::*;
use vol_per::{Container, PersState, VolPers, VolumeConverter};

use Block;
use Volume;
use Voxel;

use std::{
    any::Any,
    cmp::Eq,
    collections::HashMap,
    hash::Hash,
    marker::PhantomData,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
    u8,
};

pub struct ChunkContainer<P> {
    payload: Option<P>,
    raw: Option<Chunk>,
    rle: Option<ChunkRle>,
    file: Option<ChunkFile>,
}

pub struct ChunkConverter {}

impl<P: Send + Sync + 'static> Container for ChunkContainer<P> {
    type VoxelType = Block;
    type Payload = P;

    fn new() -> ChunkContainer<P> {
        ChunkContainer {
            payload: None,
            raw: None,
            rle: None,
            file: None,
        }
    }

    fn contains(&self, state: PersState) -> bool {
        match state {
            PersState::Raw => self.raw.is_some(),
            PersState::Rle => self.rle.is_some(),
            PersState::File => self.file.is_some(),
        }
    }

    fn insert<V: Volume<VoxelType = Block>>(&mut self, mut vol: V, state: PersState) {
        match state {
            PersState::Raw => {
                let raw: &mut Chunk = vol.as_any().downcast_mut::<Chunk>().expect("Should be Chunk");
                self.raw = Some(raw.clone());
            },
            PersState::Rle => {
                let rle: &mut ChunkRle = vol.as_any().downcast_mut::<ChunkRle>().expect("Should be ChunkRle");
                self.rle = Some(rle.clone());
            },
            PersState::File => {
                let file: &mut ChunkFile = vol.as_any().downcast_mut::<ChunkFile>().expect("Should be ChunkFile");
                self.file = Some(file.clone());
            },
        }
    }

    fn get<'a>(&'a self, state: PersState) -> Option<&'a dyn Volume<VoxelType = Block>> {
        return match state {
            PersState::Raw => self.raw.as_ref().map(|c| c as &dyn Volume<VoxelType = Block>),
            PersState::Rle => self.rle.as_ref().map(|c| c as &dyn Volume<VoxelType = Block>),
            PersState::File => self.file.as_ref().map(|c| c as &dyn Volume<VoxelType = Block>),
        };
    }

    fn get_mut<'a>(&'a mut self, state: PersState) -> Option<&'a mut dyn Volume<VoxelType = Block>> {
        return match state {
            PersState::Raw => self.raw.as_mut().map(|c| c as &mut dyn Volume<VoxelType = Block>),
            PersState::Rle => self.rle.as_mut().map(|c| c as &mut dyn Volume<VoxelType = Block>),
            PersState::File => self.file.as_mut().map(|c| c as &mut dyn Volume<VoxelType = Block>),
        };
    }

    fn payload<'a>(&'a self) -> &'a Option<P> { &self.payload }

    fn payload_mut<'a>(&'a mut self) -> &'a mut Option<P> { &mut self.payload }
}

impl<P: Send + Sync + 'static> VolumeConverter<ChunkContainer<P>> for ChunkConverter {
    fn convert(container: &mut ChunkContainer<P>, state: PersState) {
        match state {
            PersState::Raw => {
                if let Some(rle) = container.get_mut(PersState::Rle) {
                    let from: &mut ChunkRle = rle.as_any().downcast_mut::<ChunkRle>().expect("Should be ChunkRle");
                    let size = from.size();
                    let mut raw = Chunk::new();
                    raw.set_size(size);
                    let ref voxels = from.voxels_mut();
                    for x in 0..size.x {
                        for y in 0..size.y {
                            let mut old_z: i64 = 0;
                            let ref stack = voxels[(x * size.y + y) as usize];
                            for b in stack {
                                let new_z = old_z + (b.num_minus_one + 1) as i64;
                                for z in old_z..new_z {
                                    let pos = Vec3::<i64>::new(x, y, z as i64);
                                    raw.set(pos, b.block);
                                }
                                old_z = new_z;
                            }
                            for z in old_z..size.z {
                                let pos = Vec3::<i64>::new(x, y, z as i64);
                                raw.set(pos, Block::empty());
                            }
                        }
                    }
                    container.insert(raw, PersState::Raw);
                }

                // Rle -> Raw
                // File -> Rle -> Raw
            },
            PersState::Rle => {
                if let Some(raw) = container.get_mut(PersState::Raw) {
                    let from: &mut Chunk = raw.as_any().downcast_mut::<Chunk>().expect("Should be Chunk");
                    let size = from.size();
                    let mut rle = ChunkRle::new();
                    rle.set_size(size);
                    let ref mut voxels = rle.voxels_mut();
                    for x in 0..size.x {
                        for y in 0..size.y {
                            let mut old_z: i64 = 0;
                            let ref mut xy = voxels[(x * size.y + y) as usize];
                            xy.clear();
                            let mut last_block = from.at(Vec3::new(x, y, 0)).unwrap().material();
                            //println!("start pillar {}/{}", x,y);
                            for z in 1..size.z {
                                let block = from.at(Vec3::new(x, y, z)).unwrap().material();
                                //println!("block: {:?}, last_block {:?}, z {}, old_z {}", block, last_block, z, old_z);
                                if block != last_block {
                                    let zcnt = z - old_z;
                                    old_z = z;
                                    let high = ((zcnt as f32) / (BLOCK_RLE_MAX_CNT as f32 + 1.0)).ceil() as usize;
                                    let lastsize = zcnt % (BLOCK_RLE_MAX_CNT as i64 + 1);
                                    //println!("zcnt {} high {}", zcnt, high);
                                    for i in 0..high {
                                        //println!("add {:?}", last_block);
                                        xy.push(BlockRle::new(
                                            Block::new(last_block),
                                            if i == (high - 1) {
                                                (lastsize - 1) as u8
                                            } else {
                                                BLOCK_RLE_MAX_CNT
                                            },
                                        ));
                                    }
                                    last_block = block;
                                }
                            }
                            if old_z != size.z && last_block != Block::empty().material() {
                                //println!("END last_block {:?}, old_z {}", last_block, old_z);
                                let zcnt = size.z - old_z;
                                let high = ((zcnt as f32) / (BLOCK_RLE_MAX_CNT as f32 + 1.0)).ceil() as usize;
                                let lastsize = zcnt % (BLOCK_RLE_MAX_CNT as i64 + 1);
                                //println!("zcnt {} high {}", zcnt, high);
                                for i in 0..high {
                                    //println!("add {:?}", last_block);
                                    xy.push(BlockRle::new(
                                        Block::new(last_block),
                                        if i == (high - 1) {
                                            (lastsize - 1) as u8
                                        } else {
                                            BLOCK_RLE_MAX_CNT
                                        },
                                    ));
                                }
                            }
                            //println!("pillar done");
                        }
                    }
                    container.insert(rle, PersState::Rle);
                }
                let raw = container.get_mut(PersState::Raw);
                let rle = container.get_mut(PersState::Rle);
                // Raw -> Rle
                // File -> Rle
            },
            PersState::File => {
                // Rle -> File
                // Raw -> Rle -> File
            },
        }
    }
}
