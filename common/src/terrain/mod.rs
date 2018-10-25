pub mod chunk;
mod entity;
pub mod figure;
mod vol_gen;
mod vol_pers;
mod chunk_mgr;

// Reexports
pub use terrain::{
    entity::Entity,
    vol_gen::{FnGenFunc, VolGen},
    chunk_mgr::{ChunkMgr},
    vol_pers::VolPers,
};

// Standard
use std::{any::Any, cmp::Eq, fmt::Debug, hash::Hash};

// Library
use num::{Num, ToPrimitive};
use vek::*;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use bincode;
use serde::{de::DeserializeOwned, Serialize};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum PersState {
    Homo,
    Hetero,
    Rle,
}

pub trait Key: Copy + Eq + Hash + Debug + 'static {
    fn print(&self) -> String;
}

pub trait Voxel: Copy + Clone + Any {
    type Material: Copy + Clone;
    fn empty() -> Self;
    fn new(mat: Self::Material) -> Self;
    fn is_solid(&self) -> bool;
    fn material(&self) -> Self::Material;
}

pub type VoxelRelType = u16;
pub type VoxelAbsType = i64;
pub type VolumeIdxType = i32;
/// Relative VoxelId inside a volume (e.g. chunk), only posiive values
pub type VoxelRelVec = Vec3<VoxelRelType>;
/// Absolute VoxelId, this is unique for every Voxel (e.g. Block) - signed int
pub type VoxelAbsVec = Vec3<VoxelAbsType>;
/// Key of every Chunk - signed int
pub type VolumeIdxVec = Vec3<VolumeIdxType>;

pub fn volidx_to_voxabs(volidx: VolumeIdxVec, vol_size: VoxelRelVec) -> VoxelAbsVec {
    volidx.map(|e| e as i64) * vol_size.map(|e| e as i64)
}

pub fn voxabs_to_volidx(voxabs: VoxelAbsVec, vol_size: VoxelRelVec) -> VolumeIdxVec {
    voxabs.map2(vol_size, |a,s| a.div_euc(s as i64) as i32)
}

pub fn voxabs_to_voxrel(voxabs: VoxelAbsVec, vol_size: VoxelRelVec) -> VoxelRelVec {
    voxabs.map2(vol_size, |a,s| a.mod_euc(s as i64) as u16)
}

/// Helper function to manually validate a offset of any time and convert it
fn validate_offset<T: Num + ToPrimitive>(off: Vec3<T>, size: VoxelRelVec) -> Option<VoxelRelVec> {
    let sz = size;
    let off = off.map(|e| e.to_i64().unwrap());
    if off.x >= 0 && off.y >= 0 && off.z >= 0 && off.x < sz.x as i64 && off.y < sz.y as i64 && off.z < sz.z as i64 {
        Some(off.map(|e| e as u16))
    } else {
        None
    }
}

pub trait Volume { //Clone + Debug
    /// The type of the voxels contained within this volume.
    type VoxelType: Voxel;

    /// Return the size of the volume (i.e: the number of voxels on each edge).
    fn size(&self) -> VoxelRelVec;
}

pub trait ReadVolume: Volume {
    /// Return a clone of the voxel at the specified offset into the volume.
    fn at(&self, off: VoxelRelVec) -> Option<Self::VoxelType> { // Default implementation
        if let Some(off) = validate_offset(off, self.size()) {
            Some(self.at_unsafe(off))
        } else {
            None
        }
    }

    /// like `at` but acceps i64 instead of VoxelRelType
    fn at_conv(&self, off: Vec3<i64>) -> Option<Self::VoxelType> { // Default implementation
        if let Some(off) = validate_offset(off, self.size()) {
            self.at(off)
        } else {
            None
        }
    }

    /// like `at` but without any checks
    fn at_unsafe(&self, off: VoxelRelVec) -> Self::VoxelType;
}

pub trait ReadWriteVolume: ReadVolume {
    /// Replace the voxel at the specified offset into the volume, returning the old voxel if any.
    fn replace_at(&mut self, off: VoxelRelVec, vox: Self::VoxelType) -> Option<Self::VoxelType> { // Default implementation
        if let Some(off) = validate_offset(off, self.size()) {
            Some(self.replace_at_unsafe(off, vox))
        } else {
            None
        }
    }

    /// like `replace_at` but without any checks
    fn replace_at_unsafe(&mut self, off: VoxelRelVec, vox: Self::VoxelType) -> Self::VoxelType;

    /// Set the voxel at the specified offset into the volume
    fn set_at(&mut self, off: VoxelRelVec, vox: Self::VoxelType) { // Default implementation
        let _ = self.replace_at(off, vox);
    }

    /// like `set_at` but without any checks
    fn set_at_unsafe(&mut self, off: VoxelRelVec, vox: Self::VoxelType) { // Default implementation
        let _ = self.replace_at_unsafe(off, vox);
    }

    /// Set every voxel, empty or otherwise, to the given voxel type.
    fn fill(&mut self, vox: Self::VoxelType);
}

pub trait ConstructVolume: Volume {
    /// Construct a new empty volume with the given size.
    fn empty(size: VoxelRelVec) -> Self;

    /// Construct a new volume with the given size, filled with clones of the given voxel.
    fn filled(size: VoxelRelVec, vox: Self::VoxelType) -> Self;
}

pub trait AnyVolume: Any + Debug {
    fn as_any_mut(&mut self) -> &mut Any;
    fn as_any(&self) -> &Any;
}

impl<V: Any + Debug> AnyVolume for V where V: Clone {
    fn as_any_mut(&mut self) -> &mut Any { self }
    fn as_any(&self) -> &Any { self }
}

pub trait ConvertVolume: Volume + Clone + Debug {
    fn convert<VC: VolCluster<VoxelType = Self::VoxelType>>(&self, state: PersState, con: &mut VC);
}

pub trait SerializeVolume: Volume {
    fn to_bytes(&self) -> Result<Vec<u8>, ()>;
    fn from_bytes(data: &[u8]) -> Result<Self, ()> where Self: Sized;
}

impl<V: Volume> SerializeVolume for V where V: Serialize + DeserializeOwned {
    fn to_bytes(&self) -> Result<Vec<u8>, ()> { bincode::serialize(&self).map_err(|_e| ()) }

    fn from_bytes(data: &[u8]) -> Result<Self, ()> where Self: Sized {
        bincode::deserialize(data).map_err(|_e| ())
    }
}

pub trait VolCluster: Send + Sync + 'static {
    type VoxelType: Voxel;

    //fn new() -> Self;
    fn contains(&self, state: PersState) -> bool;
    //fn empty(&self) -> bool;
    fn insert<V: Volume<VoxelType = Self::VoxelType> + AnyVolume>(&mut self, vol: V);
    fn remove(&mut self, state: PersState);
    fn get<'a>(&'a self, state: PersState) -> Option<&'a dyn ReadVolume<VoxelType = Self::VoxelType>>;
    fn get_mut<'a>(&'a mut self, state: PersState) -> Option<&'a mut dyn ReadWriteVolume<VoxelType = Self::VoxelType>>;
    fn get_vol<'a>(&'a self, state: PersState) -> Option<&'a dyn Volume<VoxelType = Self::VoxelType>>;
    fn get_serializeable<'a>(&'a self, state: PersState) -> Option<&'a dyn SerializeVolume<VoxelType = Self::VoxelType>>;
    fn get_any<'a>(&'a self, state: PersState) -> Option<&'a dyn AnyVolume>;
    //fn prefered<'a>(&'a self) -> Option<&'a dyn ReadVolume<VoxelType = Self::VoxelType>>;
    //fn prefered_mut<'a>(&'a mut self) -> Option<&'a mut dyn ReadWriteVolume<VoxelType = Self::VoxelType>>;
    //fn prefered_vol<'a>(&'a self) -> Option<&'a dyn Volume<VoxelType = Self::VoxelType>>;
    //fn prefered_serializeable<'a>(&'a self) -> Option<&'a dyn SerializeVolume<VoxelType = Self::VoxelType>>;
    //fn prefered_any<'a>(&'a self) -> Option<&'a dyn AnyVolume>;
}

pub trait PhysicalVolume: Volume {
    fn scale(&self) -> Vec3<f32> {
        // Default implementation
        Vec3 { x: 1.0, y: 1.0, z: 1.0 }
    }
}

pub trait Container {
   type Payload;
   type Cluster: VolCluster;

   fn payload(&self) -> RwLockReadGuard<Option<Self::Payload>>;
   fn payload_mut(&self) -> RwLockWriteGuard<Option<Self::Payload>>;
   fn payload_try(&self) -> Option<RwLockReadGuard<Option<Self::Payload>>>;
   fn payload_try_mut(&self) -> Option<RwLockWriteGuard<Option<Self::Payload>>>;
   fn data(&self) -> RwLockReadGuard<Self::Cluster>;
   fn data_mut(&self) -> RwLockWriteGuard<Self::Cluster>;
   fn data_try(&self) -> Option<RwLockReadGuard<Self::Cluster>>;
   fn data_try_mut(&self) -> Option<RwLockWriteGuard<Self::Cluster>>;
}
