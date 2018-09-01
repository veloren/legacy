use Volume;
use Voxel;

use std::{thread, time};

use std::{
    cmp::Eq,
    fmt::Debug,
    collections::HashMap,
    hash::Hash,
    marker::PhantomData,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
    time::{SystemTime, UNIX_EPOCH},
};

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

pub trait Key: Copy + Eq + Hash + Debug + 'static {
    fn print(&self) -> String;
}

pub trait VolContainer: Send + Sync + 'static {
    type VoxelType: Voxel;

    fn new() -> Self;
    fn contains(&self, state: PersState) -> bool;
    fn insert<V: Volume<VoxelType = Self::VoxelType>>(&mut self, vol: V, state: PersState);
    fn remove(&mut self, state: PersState);
    fn get<'a>(&'a self, state: PersState) -> Option<&'a dyn Volume<VoxelType = Self::VoxelType>>;
    fn get_mut<'a>(&'a mut self, state: PersState) -> Option<&'a mut dyn Volume<VoxelType = Self::VoxelType>>;
}

pub struct Container<C: VolContainer, P: Send + Sync + 'static> {
    last_access: RwLock<SystemTime>,
    payload: RwLock<Option<P>>,
    vols: RwLock<C>,
}

impl<C: VolContainer, P: Send + Sync + 'static> Container<C, P> {
    pub fn new() -> Container<C, P> {
        Container {
            last_access: RwLock::new(SystemTime::now()),
            payload: RwLock::new(None),
            vols: RwLock::new(C::new()),
        }
    }

    pub fn payload(&self) -> RwLockReadGuard<Option<P>> { self.payload.read().unwrap() }

    pub fn payload_mut(&self) -> RwLockWriteGuard<Option<P>> { self.payload.write().unwrap() }

    pub fn vols(&self) -> RwLockReadGuard<C> { self.vols.read().unwrap() }

    pub fn vols_mut(&self) -> RwLockWriteGuard<C> { self.vols.write().unwrap() }

    pub fn last_access(&self) -> RwLockReadGuard<SystemTime> { self.last_access.read().unwrap() }

    pub fn set_access(&self) { *self.last_access.write().unwrap() = SystemTime::now(); }
}

/*
pub trait Container: Send + Sync + 'static {
    type VoxelType: Voxel;
    type Payload;

    fn new() -> Self;
    fn contains(&self, state: PersState) -> bool;
    fn insert<V: Volume<VoxelType = Self::VoxelType>>(&mut self, vol: V, state: PersState);
    fn drop(&mut self, state: PersState);
    fn get<'a>(&'a self, state: PersState) -> Option<&'a dyn Volume<VoxelType = Self::VoxelType>>;
    fn get_mut<'a>(&'a mut self, state: PersState) -> Option<&'a mut dyn Volume<VoxelType = Self::VoxelType>>;
    fn payload<'a>(&'a self) -> &'a Option<Self::Payload>;
    fn payload_mut<'a>(&'a mut self) -> &'a mut Option<Self::Payload>;
}*/

pub trait VolumeConverter<C: VolContainer> {
    fn convert<K: Key>(key: &K, container: &mut C, state: PersState);
}

pub struct VolPers<K: Key, C: VolContainer, VC: VolumeConverter<C>, P: Send + Sync + 'static> {
    data: RwLock<HashMap<K, Arc<Container<C, P>>>>,
    phantom: PhantomData<VC>,
}

impl<K: Key, C: VolContainer, VC: VolumeConverter<C>, P: Send + Sync + 'static> VolPers<K, C, VC, P> {
    pub fn new() -> VolPers<K, C, VC, P> {
        VolPers {
            data: RwLock::new(HashMap::new()),
            phantom: PhantomData,
        }
    }

    pub fn data_mut(&self) -> RwLockWriteGuard<HashMap<K, Arc<Container<C, P>>>> { self.data.write().unwrap() }

    pub fn data(&self) -> RwLockReadGuard<HashMap<K, Arc<Container<C, P>>>> { self.data.read().unwrap() }

    pub fn get(&self, key: &K) -> Option<Arc<Container<C, P>>> {
        self.data.read().unwrap().get(&key).map(|v| {
            v.clone()
        })
    }

    pub fn exists(&self, key: &K, state: PersState) -> bool {
        if let Some(entry) = self.data.read().unwrap().get(&key) {
            return entry.vols.read().unwrap().contains(state);
        }
        return false;
    }

    pub fn generate(&self, key: &K, state: PersState) {
        let x = self.data.read().unwrap().get(&key).map(|v| v.clone());
        if let Some(container) = x {
            container.set_access();
            let mut lock = container.vols_mut();
            let contains = lock.contains(state.clone());
            if !contains {
                info!("generate from persistence key: {:?} state: {:?}", key, state);
                println!("contains file: {}", lock.contains(PersState::File));
                VC::convert(key, &mut lock, state);
            }
        }
    }

    pub fn offload(&self) {
        let now = SystemTime::now();
        let mut offload_queue = vec!(); //we allocate in a container here, to reduce lock phase
        for (key, container) in self.data.read().unwrap().iter() {
            let diff;
            let contains;
            {
                diff = now.duration_since(*container.last_access());
                let lock = container.vols();
                contains = lock.contains(PersState::Raw) || lock.contains(PersState::Rle);
            }
            if let Ok(diff) = diff {
                if diff.as_secs() > 5 && contains {
                    info!("drop persistence to Rle: {:?} after {}", key, diff.as_secs());
                    offload_queue.push(((*key).clone(), container.clone()));
                }
            }
        }
        for (key, container) in offload_queue.iter() {
            let mut lock = container.vols_mut();
            VC::convert(key, &mut lock, PersState::File);
            println!("offload contains file: {}", lock.contains(PersState::File));
            lock.remove(PersState::Raw);
            lock.remove(PersState::Rle);
            *container.payload_mut() = None;
        }
        info!("DONE DONE");
    }
}
