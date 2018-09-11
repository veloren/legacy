// Standard
use std::{
    cmp::Eq,
    collections::HashMap,
    hash::Hash,
    marker::PhantomData,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

// Local
use Volume;
use Voxel;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum PersState {
    Raw,
    Rle,
    File,
    //Network,
}

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

pub trait Container: Send + Sync + 'static {
    type VoxelType: Voxel;
    type Payload;

    fn new() -> Self;
    fn contains(&self, state: PersState) -> bool;
    fn insert<V: Volume<VoxelType = Self::VoxelType>>(&mut self, vol: V, state: PersState);
    fn get<'a>(&'a self, state: PersState) -> Option<&'a dyn Volume<VoxelType = Self::VoxelType>>;
    fn get_mut<'a>(&'a mut self, state: PersState) -> Option<&'a mut dyn Volume<VoxelType = Self::VoxelType>>;
    fn payload<'a>(&'a self) -> &'a Option<Self::Payload>;
    fn payload_mut<'a>(&'a mut self) -> &'a mut Option<Self::Payload>;
}

pub trait VolumeConverter<C: Container> {
    fn convert(container: &mut C, state: PersState);
}

pub struct VolPers<K: Eq + Hash + 'static, C: Container, VC: VolumeConverter<C>> {
    data: RwLock<HashMap<K, Arc<RwLock<C>>>>,
    phantom: PhantomData<VC>,
}

impl<K: Eq + Hash + 'static, C: Container, VC: VolumeConverter<C>> VolPers<K, C, VC> {
    pub fn new() -> VolPers<K, C, VC> {
        VolPers {
            data: RwLock::new(HashMap::new()),
            phantom: PhantomData,
        }
    }

    pub fn data_mut(&self) -> RwLockWriteGuard<HashMap<K, Arc<RwLock<C>>>> { self.data.write().unwrap() }

    pub fn data(&self) -> RwLockReadGuard<HashMap<K, Arc<RwLock<C>>>> { self.data.read().unwrap() }

    pub fn get(&self, key: &K) -> Option<Arc<RwLock<C>>> { self.data.read().unwrap().get(&key).map(|v| v.clone()) }

    pub fn exists(&self, key: &K, state: PersState) -> bool {
        if let Some(x) = self.data.read().unwrap().get(&key) {
            return x.read().unwrap().contains(state);
        }
        return false;
    }

    pub fn generate(&self, key: &K, state: PersState) {
        let x = self.data.read().unwrap().get(&key).map(|v| v.clone());
        if let Some(x) = x {
            let mut con = x.write().unwrap();
            let contains = con.contains(state.clone());
            if !contains {
                VC::convert(&mut con, state);
            }
        }
    }
}
