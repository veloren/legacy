// Library
use vek::*;

// Local
use chunk::Chunk;
use chunk_file::ChunkFile;
use chunk_rle::ChunkRle;
use vol_per::{Container, PersState, VolPers, VolumeConverter};
use Block;
use Volume;
use Voxel;

use std::{any::Any, cmp::Eq, collections::HashMap, hash::Hash, marker::PhantomData, sync::Arc, u8};

use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

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
                            let old_z = 0;
                            let ref stack = voxels[x as usize][y as usize];
                            for b in stack {
                                let new_z = old_z + b.num;
                                for z in old_z..new_z {
                                    let pos = Vec3::<i64>::new(x, y, z as i64);
                                    raw.set(pos, b.block);
                                }
                            }
                        }
                    }
                    let ref mut con_vol = container.get_mut(PersState::Raw);
                    *con_vol = Some(&mut raw);
                }

                // Rle -> Raw
                // File -> Rle -> Raw
            },
            PersState::Rle => {
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
