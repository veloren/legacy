// Library
use vek::*;

// Local
use terrain::{chunk::Block, ConstructVolume, PhysicalVolume, ReadVolume, ReadWriteVolume, Volume, VoxRel, Voxel};

#[derive(Clone, Debug, PartialEq)]
pub struct HeterogeneousData {
    size: Vec3<VoxRel>,
    voxels: Vec<Block>,
}

impl HeterogeneousData {
    fn calculate_index(&self, off: Vec3<VoxRel>) -> usize {
        (off.x as usize * self.size.y as usize * self.size.z as usize
            + off.y as usize * self.size.z as usize
            + off.z as usize)
    }

    pub(crate) fn voxels_mut(&mut self) -> &mut Vec<Block> { &mut self.voxels }
}

impl Volume for HeterogeneousData {
    type VoxelType = Block;

    fn size(&self) -> Vec3<VoxRel> { self.size }
}

impl ReadVolume for HeterogeneousData {
    fn at_unchecked(&self, off: Vec3<VoxRel>) -> Block { self.voxels[self.calculate_index(off)] }
}

impl ReadWriteVolume for HeterogeneousData {
    fn replace_at_unchecked(&mut self, off: Vec3<VoxRel>, vox: Self::VoxelType) -> Self::VoxelType {
        let i = self.calculate_index(off);
        let r = self.voxels[i];
        self.voxels[i] = vox;
        r
    }

    fn fill(&mut self, vox: Self::VoxelType) {
        for v in self.voxels.iter_mut() {
            *v = vox;
        }
    }
}

impl ConstructVolume for HeterogeneousData {
    fn filled(size: Vec3<VoxRel>, vox: Self::VoxelType) -> HeterogeneousData {
        HeterogeneousData {
            size,
            voxels: vec![vox; size.map(|e| e as usize).product()],
        }
    }

    fn empty(size: Vec3<VoxRel>) -> HeterogeneousData { Self::filled(size, Block::empty()) }
}

impl PhysicalVolume for HeterogeneousData {}
