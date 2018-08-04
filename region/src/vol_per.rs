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

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum VolState {
    Raw,
    Rle,
    File,
    //Network,
}
const NUMBER_OF_ELEMENTS_IN_VOLSTATE: usize = 3; //TODO: really rust ?

/*
 How persistence works:
 Persistents takes care of the act of storing Volumes.
 A persistence can store multiple structs which implement Volume and have the same VoxelType.
 The Idea is, that you give Persistence a volume to store and it will take care that this volume is available when needed.
 You then can request any Volume in any VolState and Persistence will convert it from any other state.
 It needs to have a Volume it atleast one state for this conversion to work.
 It tries to cache specific often needed Volumes in it's memory and can "reduce the VolState", e.g. store a VOlume to file if it's not used for a while
 When it's requiered again it will be reloaded into the requiered state.

 When accessing a Chunk you get access to all States of a Chunk.
 When you modify a version, you must either also change all other implementations or drop them!
*/

/*
pub trait CommonVolume<VT: Voxel> : Volume<VoxelType=VT> {

}
*/

pub struct Container<VT: Voxel, P: Send + Sync + 'static> {
    payload: P,
    dummy: VT,
    states: [Option<Box<Volume<VoxelType=VT>>>; NUMBER_OF_ELEMENTS_IN_VOLSTATE],
}

pub trait VolumeConverter<VT: Voxel> {
    fn convert<P: Send + Sync + 'static>(container: &mut Container<VT, P>, state: VolState);
}

pub struct VolPers<K: Eq + Hash + 'static, VT: Voxel, VC: VolumeConverter<VT>, P: Send + Sync + 'static> {
    data: RwLock<HashMap<K, Arc<RwLock<Container<VT, P>>>>>,
    dummy: Option<VC>,
}

impl<VT: Voxel, P: Send + Sync + 'static> Container<VT, P> {
    pub fn exists(&self, state: VolState) -> bool {
        match state {
            VolState::Raw => self.states[0].is_some(),
            VolState::Rle => self.states[1].is_some(),
            VolState::File => self.states[2].is_some(),
        }
    }

    pub fn get<'a>(&'a self, state: VolState) -> &'a Option<Box<Volume<VoxelType=VT> + 'static>> {
        match state {
            VolState::Raw => &self.states[0],
            VolState::Rle => &self.states[1],
            VolState::File => &self.states[2],
        }
    }
}

impl<K: Eq + Hash + 'static, VT: Voxel, VC: VolumeConverter<VT>, P: Send + Sync + 'static> VolPers<K, VT, VC, P> {
    pub fn new() -> VolPers<K, VT, VC, P> {
        VolPers {
            data: RwLock::new(HashMap::new()),
            dummy: None,
        }
    }

    pub fn get(&self, key: &K) -> Option<Arc<RwLock<Container<VT, P>>>> {
        self.data.read().unwrap().get(&key).map(|v| v.clone())
    }

    pub fn exists(&self, key: &K, state: VolState) -> bool{
        let x = self.data.read().unwrap().get(&key).map(|v| v.clone());
        if let Some(x) = x {
            let con = x.read().unwrap();
            let contains = con.exists(state);
            return contains;
        }
        return false;
    }

    pub fn generate(&self, key: &K, state: VolState) {
        let x = self.data.read().unwrap().get(&key).map(|v| v.clone());
        if let Some(x) = x {
            let mut con = x.write().unwrap();
            let contains = con.exists(state.clone());
            if !contains {
                //TODO: does this logic belong in this class or in the converter ???
                VC::convert(&mut con, state);
            }
        }
    }
}
