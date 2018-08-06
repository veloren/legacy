use Volume;
use Voxel;

use std::{
    cmp::Eq,
    collections::HashMap,
    hash::Hash,
    marker::PhantomData,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum PersState {
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
 You then can request any Volume in any PersState and Persistence will convert it from any other state.
 It needs to have a Volume it atleast one state for this conversion to work.
 It tries to cache specific often needed Volumes in it's memory and can "reduce the PersState", e.g. store a VOlume to file if it's not used for a while
 When it's requiered again it will be reloaded into the requiered state.

 When accessing a Chunk you get access to all States of a Chunk.
 When you modify a version, you must either also change all other implementations or drop them!
*/

/*
pub trait CommonVolume<VT: Voxel> : Volume<VoxelType=VT> {

}
*/

pub struct Container<VT: Voxel, P: Send + Sync + 'static> {
    payload: Option<P>,
    states: [Option<Box<Volume<VoxelType = VT>>>; NUMBER_OF_ELEMENTS_IN_VOLSTATE],
}

pub trait VolumeConverter<VT: Voxel>: Send + Sync + 'static {
    fn convert<P: Send + Sync + 'static>(container: &mut Container<VT, P>, state: PersState);
}

pub struct VolPers<K: Eq + Hash + 'static, VT: Voxel, VC: VolumeConverter<VT>, P: Send + Sync + 'static> {
    data: RwLock<HashMap<K, Arc<RwLock<Container<VT, P>>>>>,
    phantom: PhantomData<VC>,
}

impl<VT: Voxel, P: Send + Sync + 'static> Container<VT, P> {
    pub fn new() -> Container<VT, P> {
        Container {
            payload: None,
            states: [None, None, None], // this needs no Copy trait
        }
    }

    pub fn exists(&self, state: PersState) -> bool {
        match state {
            PersState::Raw => self.states[0].is_some(),
            PersState::Rle => self.states[1].is_some(),
            PersState::File => self.states[2].is_some(),
        }
    }

    pub fn get(&self, state: PersState) -> &Option<Box<Volume<VoxelType = VT> + 'static>> {
        match state {
            PersState::Raw => &self.states[0],
            PersState::Rle => &self.states[1],
            PersState::File => &self.states[2],
        }
    }

    pub fn get_mut(&mut self, state: PersState) -> &mut Option<Box<Volume<VoxelType = VT> + 'static>> {
        match state {
            PersState::Raw => &mut self.states[0],
            PersState::Rle => &mut self.states[1],
            PersState::File => &mut self.states[2],
        }
    }

    pub fn payload<'a>(&'a self) -> &'a Option<P> { &self.payload }

    pub fn payload_mut<'a>(&'a mut self) -> &'a mut Option<P> { &mut self.payload }
}

impl<K: Eq + Hash + 'static, VT: Voxel, VC: VolumeConverter<VT>, P: Send + Sync + 'static> VolPers<K, VT, VC, P> {
    pub fn new() -> VolPers<K, VT, VC, P> {
        VolPers {
            data: RwLock::new(HashMap::new()),
            phantom: PhantomData,
        }
    }

    pub fn data_mut(&self) -> RwLockWriteGuard<HashMap<K, Arc<RwLock<Container<VT, P>>>>> { self.data.write().unwrap() }

    pub fn data(&self) -> RwLockReadGuard<HashMap<K, Arc<RwLock<Container<VT, P>>>>> { self.data.read().unwrap() }

    pub fn get(&self, key: &K) -> Option<Arc<RwLock<Container<VT, P>>>> {
        self.data.read().unwrap().get(&key).map(|v| v.clone())
    }

    pub fn exists(&self, key: &K, state: PersState) -> bool {
        if let Some(x) = self.data.read().unwrap().get(&key) {
            let con = x.read().unwrap();
            let contains = con.exists(state);
            return contains;
        }
        return false;
    }

    pub fn generate(&self, key: &K, state: PersState) {
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
