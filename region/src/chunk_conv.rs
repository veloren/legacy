use coord::prelude::*;
use vol_per::{Container, PersState, VolPers, VolumeConverter};

use Block;
use Volume;
use Voxel;

use std::{
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
                // Rle -> Raw
                // File -> Rle -> Raw
            },
            PersState::Rle => {
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
