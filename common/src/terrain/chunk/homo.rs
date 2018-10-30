// Library
use vek::*;

// Local
use terrain::{chunk::Block, ConstructVolume, PhysicalVolume, ReadVolume, Volume, VoxRel, Voxel};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct HomogeneousData {
    size: Vec3<VoxRel>,
    voxel: Block,
}

impl HomogeneousData {
    pub(crate) fn voxel_mut(&mut self) -> &mut Block { &mut self.voxel }
}

impl Volume for HomogeneousData {
    type VoxelType = Block;

    fn size(&self) -> Vec3<VoxRel> { self.size }
}

impl ReadVolume for HomogeneousData {
    fn at_unchecked(&self, _off: Vec3<VoxRel>) -> Block { self.voxel }
}

impl ConstructVolume for HomogeneousData {
    fn filled(size: Vec3<VoxRel>, vox: Self::VoxelType) -> HomogeneousData { HomogeneousData { size, voxel: vox } }

    fn empty(size: Vec3<VoxRel>) -> HomogeneousData { Self::filled(size, Block::empty()) }
}

impl PhysicalVolume for HomogeneousData {}
