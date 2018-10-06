mod block;
mod chunk_mgr;
mod raw_chunk;
#[cfg(test)]
mod tests;

// Standard
use std::fmt::Debug;

// Library
use vek::*;

pub trait Voxel: Clone + Debug + Eq {
    /// Return a variant of the voxel that may be considered to be empty or vacant.
    fn empty() -> Self;

    /// Return any variant of the voxel that may be considered solid. Primary used for testing.
    fn solid() -> Self;

    // Return true if the voxel is considered empty or vacant
    fn is_empty(&self) -> bool;
}

pub trait Volume: Clone + Debug {
    /// The type of the voxels contained within this volume.
    type Voxel: Voxel;

    /// Return the size of the volume (i.e: the number of voxels on each edge).
    fn get_size(&self) -> Vec3<u64>;

    /// The exact implementation of this function is left undefined, but it usually acts as a way
    /// to allow the volume a chance to rearrange its internal data structures into a more
    /// efficient configuration. Generally speaking, implementors should not perform any action
    /// within this method that may change the result of subsequent well-defined method calls (i.e:
    /// the apparent structure of the volume should not change)
    fn maintain(&mut self) {}
}

pub trait ReadVolume: Volume {
    /// Return a clone of the voxel at the specified offset into the volume.
    fn get_at(&self, off: Vec3<u64>) -> Option<Self::Voxel>;

    /// Return true if the volume is composed of a homogeneous voxel type.
    /// *Note that this method is permitted to return false even in cases of homogenity*
    fn is_homo(&self) -> bool { false }
}

pub trait WriteVolume: ReadVolume {
    /// Replace the voxel at the specified offset into the volume, returning the old voxel if any.
    fn replace_at(&mut self, off: Vec3<u64>, vox: Self::Voxel) -> Option<Self::Voxel>;

    /// Set the voxel at the specified offset into the volume
    fn set_at(&mut self, off: Vec3<u64>, vox: Self::Voxel) {
        // Default implementation
        let _ = self.replace_at(off, vox);
    }

    /// Remove a voxel at the specified offset into the volume, replacing it with an empty voxel.
    fn remove_at(&mut self, off: Vec3<u64>) -> Option<Self::Voxel> {
        // Default implementation
        self.replace_at(off, Self::Voxel::empty())
    }

    /// Set every voxel, empty or otherwise, to the given voxel type.
    /// *Note that this is not required to cause subsequent calls to ReadVolume::is_homo() to
    /// return true*
    fn fill(&mut self, vox: Self::Voxel) {
        // Default implementation
        let sz = self.get_size();
        for x in 0..sz.x {
            for y in 0..sz.y {
                for z in 0..sz.z {
                    self.set_at(Vec3::new(x, y, z), vox.clone());
                }
            }
        }
    }
}

pub trait ConstructVolume: Volume {
    /// Construct a new empty volume with the given size.
    fn empty(sz: Vec3<u64>) -> Self { Self::filled(sz, Self::Voxel::empty()) }

    /// Construct a new volume with the given size, filled with clones of the given voxel.
    fn filled(sz: Vec3<u64>, vox: Self::Voxel) -> Self;
}

pub trait SerializeVolume: Volume {
    //TODO
}
