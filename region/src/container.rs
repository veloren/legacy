use super::{PersState, Volume, Voxel};

use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::time::SystemTime;

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

    pub fn payload(&self) -> RwLockReadGuard<Option<P>> { self.payload.read() }

    pub fn payload_mut(&self) -> RwLockWriteGuard<Option<P>> { self.payload.write() }

    pub fn payload_try(&self) -> Option<RwLockReadGuard<Option<P>>> { self.payload.try_read() }

    pub fn payload_try_mut(&self) -> Option<RwLockWriteGuard<Option<P>>> { self.payload.try_write() }

    pub fn vols(&self) -> RwLockReadGuard<C> { self.vols.read() }

    pub fn vols_mut(&self) -> RwLockWriteGuard<C> { self.vols.write() }

    pub fn vols_try(&self) -> Option<RwLockReadGuard<C>> { self.vols.try_read() }

    pub fn last_access(&self) -> RwLockReadGuard<SystemTime> { self.last_access.read() }

    pub fn set_access(&self) { *self.last_access.write() = SystemTime::now(); }
}
