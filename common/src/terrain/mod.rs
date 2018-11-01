pub mod chunk;
mod chunk_mgr;
mod entity;
pub mod figure;
mod vol_gen;

// Reexports
pub use terrain::{
    chunk_mgr::{BlockLoader, ChunkMgr},
    entity::Entity,
    vol_gen::{FnDropFunc, FnGenFunc, VolGen},
};

// Standard
use std::{any::Any, cmp::Eq, fmt::Debug, hash::Hash};

// Library
use bincode;
use num::{Num, ToPrimitive};
use parking_lot::{RwLockReadGuard, RwLockWriteGuard};
use serde::{de::DeserializeOwned, Serialize};
use vek::*;

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

/// Relative VoxelId inside a volume (e.g. chunk), only posiive values
pub type VoxRel = u16;
/// Absolute VoxelId, this is unique for every Voxel (e.g. Block) - signed int
pub type VoxAbs = i64;
/// Key of every Chunk - signed int
pub type VolOffs = i32;

pub fn voloffs_to_voxabs(volidx: Vec3<VolOffs>, vol_size: Vec3<VoxRel>) -> Vec3<VoxAbs> {
    volidx.map(|e| e as VoxAbs) * vol_size.map(|e| e as VoxAbs)
}

pub fn voxabs_to_voloffs(voxabs: Vec3<VoxAbs>, vol_size: Vec3<VoxRel>) -> Vec3<VolOffs> {
    voxabs.map2(vol_size, |a, s| a.div_euc(s as VoxAbs) as VolOffs)
}

pub fn voxabs_to_voxrel(voxabs: Vec3<VoxAbs>, vol_size: Vec3<VoxRel>) -> Vec3<VoxRel> {
    voxabs.map2(vol_size, |a, s| a.mod_euc(s as VoxAbs) as VoxRel)
}

/// Helper function to manually validate a offset of any time and convert it
fn validate_offset<T: Num + ToPrimitive>(off: Vec3<T>, size: Vec3<VoxRel>) -> Option<Vec3<VoxRel>> {
    let off = off.map(|e| e.to_i64().unwrap());
    if off.x >= 0
        && off.y >= 0
        && off.z >= 0
        && off.x < size.x as VoxAbs
        && off.y < size.y as VoxAbs
        && off.z < size.z as VoxAbs
    {
        Some(off.map(|e| e as VoxRel))
    } else {
        None
    }
}

pub trait Volume {
    //Clone + Debug
    /// The type of the voxels contained within this volume.
    type VoxelType: Voxel;

    /// Return the size of the volume (i.e: the number of voxels on each edge).
    fn size(&self) -> Vec3<VoxRel>;
}

pub trait ReadVolume: Volume {
    /// Return a clone of the voxel at the specified offset into the volume.
    fn at(&self, off: Vec3<VoxRel>) -> Option<Self::VoxelType> {
        // Default implementation
        validate_offset(off, self.size()).map(|off| self.at_unchecked(off))
    }

    /// like `at` but acceps i64 instead of VoxRel
    fn at_conv(&self, off: Vec3<i64>) -> Option<Self::VoxelType> {
        // Default implementation
        validate_offset(off, self.size()).map(|off| self.at_unchecked(off))
    }

    /// like `at` but without any checks
    fn at_unchecked(&self, off: Vec3<VoxRel>) -> Self::VoxelType;
}

pub trait ReadWriteVolume: ReadVolume {
    /// Replace the voxel at the specified offset into the volume, returning the old voxel if any.
    fn replace_at(&mut self, off: Vec3<VoxRel>, vox: Self::VoxelType) -> Option<Self::VoxelType> {
        // Default implementation
        validate_offset(off, self.size()).map(|off| self.replace_at_unchecked(off, vox))
    }

    /// like `replace_at` but without any checks
    fn replace_at_unchecked(&mut self, off: Vec3<VoxRel>, vox: Self::VoxelType) -> Self::VoxelType;

    /// Set the voxel at the specified offset into the volume
    fn set_at(&mut self, off: Vec3<VoxRel>, vox: Self::VoxelType) {
        // Default implementation
        let _ = self.replace_at(off, vox);
    }

    /// like `set_at` but without any checks
    fn set_at_unchecked(&mut self, off: Vec3<VoxRel>, vox: Self::VoxelType) {
        // Default implementation
        let _ = self.replace_at_unchecked(off, vox);
    }

    /// Set every voxel, empty or otherwise, to the given voxel type.
    fn fill(&mut self, vox: Self::VoxelType);
}

pub trait ConstructVolume: Volume {
    /// Construct a new empty volume with the given size.
    fn empty(size: Vec3<VoxRel>) -> Self;

    /// Construct a new volume with the given size, filled with clones of the given voxel.
    fn filled(size: Vec3<VoxRel>, vox: Self::VoxelType) -> Self;
}

pub trait AnyVolume: Any + Debug {
    fn as_any_mut(&mut self) -> &mut Any;
    fn as_any(&self) -> &Any;
}

impl<V: Any + Debug> AnyVolume for V
where
    V: Clone,
{
    fn as_any_mut(&mut self) -> &mut Any { self }
    fn as_any(&self) -> &Any { self }
}

pub trait SerializeVolume: Volume {
    fn to_bytes(&self) -> Result<Vec<u8>, ()>;
    fn from_bytes(data: &[u8]) -> Result<Self, ()>
    where
        Self: Sized;
}

impl<V: Volume> SerializeVolume for V
where
    V: Volume + Serialize + DeserializeOwned,
{
    fn to_bytes(&self) -> Result<Vec<u8>, ()> { bincode::serialize(&self).map_err(|_e| ()) }

    fn from_bytes(data: &[u8]) -> Result<Self, ()>
    where
        Self: Sized,
    {
        bincode::deserialize(data).map_err(|_e| ())
    }
}

pub trait PhysicalVolume: ReadVolume {
    fn scale(&self) -> Vec3<f32> {
        // Default implementation
        Vec3 { x: 1.0, y: 1.0, z: 1.0 }
    }
}

pub trait VolCluster: Send + Sync + 'static {
    type VoxelType: Voxel;

    fn contains(&self, state: PersState) -> bool;
    fn convert(&mut self, state: PersState);
    fn insert<V: Volume<VoxelType = Self::VoxelType> + AnyVolume>(&mut self, vol: V);
    fn remove(&mut self, state: PersState);
    fn get<'a>(&'a self, state: PersState) -> Option<&'a dyn ReadVolume<VoxelType = Self::VoxelType>>;
    fn get_mut<'a>(&'a mut self, state: PersState) -> Option<&'a mut dyn ReadWriteVolume<VoxelType = Self::VoxelType>>;
    fn get_vol<'a>(&'a self, state: PersState) -> Option<&'a dyn Volume<VoxelType = Self::VoxelType>>;
    fn get_physical<'a>(&'a self, state: PersState) -> Option<&'a dyn PhysicalVolume<VoxelType = Self::VoxelType>>;
    fn get_serializeable<'a>(
        &'a self,
        state: PersState,
    ) -> Option<&'a dyn SerializeVolume<VoxelType = Self::VoxelType>>;
    fn get_any<'a>(&'a self, state: PersState) -> Option<&'a dyn AnyVolume>;
    fn prefered<'a>(&'a self) -> Option<&'a dyn ReadVolume<VoxelType = Self::VoxelType>>;
    fn prefered_mut<'a>(&'a mut self) -> Option<&'a mut dyn ReadWriteVolume<VoxelType = Self::VoxelType>>;
    fn prefered_vol<'a>(&'a self) -> Option<&'a dyn Volume<VoxelType = Self::VoxelType>>;
    fn prefered_physical<'a>(&'a self) -> Option<&'a dyn PhysicalVolume<VoxelType = Self::VoxelType>>;
    fn prefered_serializeable<'a>(&'a self) -> Option<&'a dyn SerializeVolume<VoxelType = Self::VoxelType>>;
    fn prefered_any<'a>(&'a self) -> Option<&'a dyn AnyVolume>;
    fn to_bytes(&mut self) -> Result<Vec<u8>, ()>;
    fn from_bytes(data: &[u8]) -> Result<Self, ()>
    where
        Self: Sized;
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
