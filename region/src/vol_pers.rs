use super::{Container, Key, PersState, VolContainer, VolConverter};

use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::{collections::HashMap, marker::PhantomData, sync::Arc, time::SystemTime};

/*
 How persistence works:
 Persistents takes care of the act of storing Volumes.
 A persistence can store multiple structs which implement Volume and have the same VoxelType.
 The Idea is, that you give Persistence a volume to store and it will take care that this volume is available when needed.
 You then can request any Volume in any PersState and Persistence will convert it from any other state.
 It needs to have a Volume it atleast one state for this conversion to work.
 It tries to cache specific often needed Volumes in it's memory and can "reduce the PersState", e.g. store a VOlume to file if it's not used for a while
 When it's requiered again it will be reloaded into the requiered state.
 File PersState are hold in the cold storage, while all others are hold in the hot storage to not increase the HashMap for normal usage

 When accessing a Chunk you get access to all States of a Chunk.
 When you modify a version, you must either also change all other implementations or drop them!
*/

pub struct VolPers<K: Key, C: VolContainer, VC: VolConverter<C>, P: Send + Sync + 'static> {
    hot: RwLock<HashMap<K, Arc<Container<C, P>>>>,
    cold: RwLock<HashMap<K, Arc<Container<C, P>>>>,
    phantom: PhantomData<VC>,
}

impl<K: Key, C: VolContainer, VC: VolConverter<C>, P: Send + Sync + 'static> VolPers<K, C, VC, P> {
    pub fn new() -> VolPers<K, C, VC, P> {
        VolPers {
            hot: RwLock::new(HashMap::new()),
            cold: RwLock::new(HashMap::new()),
            phantom: PhantomData,
        }
    }

    pub fn hot_mut(&self) -> RwLockWriteGuard<HashMap<K, Arc<Container<C, P>>>> { self.hot.write() }

    pub fn hot(&self) -> RwLockReadGuard<HashMap<K, Arc<Container<C, P>>>> { self.hot.read() }

    pub fn get(&self, key: &K) -> Option<Arc<Container<C, P>>> {
        let mut x = self.hot.read().get(&key).map(|v| v.clone());
        if x.is_none() {
            x = self.cold.read().get(&key).map(|v| v.clone());
        }
        return x;
    }

    pub fn exists(&self, key: &K, state: PersState) -> bool {
        if let Some(entry) = self.cold.read().get(&key) {
            return entry.vols().contains(state);
        }
        if let Some(entry) = self.hot.read().get(&key) {
            return entry.vols().contains(state);
        }
        return false;
    }

    pub fn generate(&self, key: &K, state: PersState) {
        let mut x = self.hot.read().get(&key).map(|v| v.clone());
        if x.is_none() {
            x = self.cold.read().get(&key).map(|v| v.clone());
        }
        if let Some(container) = x {
            container.set_access();
            let mut lock = container.vols_mut();
            let contains = lock.contains(state.clone());
            if !contains {
                info!("generate from persistence key: {:?} state: {:?}", key, state);
                VC::convert(key, &mut lock, state);
            }
        }
    }

    pub fn try_cold_offload(&self) {
        let h = self.hot.try_write();
        if let Some(mut h) = h {
            let mut c = self.cold.try_write();
            if let Some(ref mut c) = c {
                let mut remove_from_c = vec![];
                for (key, container) in c.iter() {
                    let v = container.vols_try();
                    if let Some(v) = v {
                        if v.contains(PersState::Raw) || v.contains(PersState::Rle) {
                            h.insert(*key, container.clone());
                            remove_from_c.push(key.clone());
                        }
                    } else {
                        println!("noooo {:?}", key);
                    }
                }
                for key in remove_from_c.iter() {
                    c.remove(key);
                }
                let mut remove_from_h = vec![];
                for (key, container) in h.iter() {
                    let v = container.vols_try();
                    if let Some(v) = v {
                        if !v.contains(PersState::Raw) && !v.contains(PersState::Rle) {
                            c.insert(*key, container.clone());
                            remove_from_h.push(key.clone());
                        }
                    } else {
                        println!("noooo {:?}", key);
                    }
                }
                for key in remove_from_h.iter() {
                    h.remove(key);
                }
            }
        }
    }

    pub fn debug(&self) {
        debug!(
            "hot containers: {}, cold containers: {}",
            self.hot.read().len(),
            self.cold.read().len()
        );
    }
}
