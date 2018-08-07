use chunk::Chunk;
use chunk_file::ChunkFile;
use chunk_rle::ChunkRle;
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

pub struct ChunkConverter<VT: Voxel> {
    dummy: VT,
}

impl VolumeConverter<Block> for ChunkConverter<Block> {
    fn convert<P: Send + Sync + 'static>(container: &mut Container<Block, P>, state: PersState) {
        match state {
            PersState::Raw => {
                if let Some(rle) = container.get_mut(PersState::Rle) {
                    let s = rle.size();
                    let from: &mut ChunkRle = match rle.as_any().downcast_mut::<ChunkRle>() {
                        Some(mut b) => b,
                        None => panic!("Should be ChunkRle"),
                    };
                    let mut raw = Chunk::new();
                    raw.set_size(s);
                    let ref from_voxels = from.voxels_mut();
                    //let ref raw_voxels = raw.voxels_mut();
                    for x in 0..s.x {
                        for y in 0..s.y {
                            let mut old_z = 0;
                            let ref stack = from_voxels[x as usize][y as usize];
                            for b in stack {
                                let new_z = old_z + b.cnt;
                                for z in old_z..new_z {
                                    let pos = Vec3::<i64>::new(x, y, z as i64);
                                    raw.set(pos, b.block);
                                }
                            }
                        }
                    }
                    *container.get_mut(PersState::Raw) = Some(Box::new(raw));
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
