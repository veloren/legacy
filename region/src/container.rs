use super::{PersState, Volume, Voxel};

use std::{
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
    time::SystemTime,
};

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
