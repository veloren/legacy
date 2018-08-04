use vol_per::{VolumeConverter, VolPers, Container, VolState};
use coord::prelude::*;

use Block;
use Volume;
use Voxel;

use std::collections::HashMap;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::marker::PhantomData;
use std::u8;
use std::cmp::Eq;
use std::hash::Hash;

pub struct ChunkConverter<VT: Voxel> {
    dummy: VT,
}

impl VolumeConverter<Block> for ChunkConverter<Block> {
    fn convert<P: Send + Sync + 'static>(container: &mut Container<Block, P>, state: VolState) {
        match state {
            VolState::Raw => {
                // Rle -> Raw
                // File -> Rle -> Raw
            },
            VolState::Rle => {
                // Raw -> Rle
                // File -> Rle
            },
            VolState::File => {
                // Rle -> File
                // Raw -> Rle -> File
            },
        }
    }
}
