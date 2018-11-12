// Library
use vek::*;

// Local
use terrain::{
    figure::{Cell, CellMaterial},
    ConstructVolume, PhysicalVolume, ReadVolume, ReadWriteVolume, Volume, VoxRel, Voxel,
    chunk::Block,
};

pub struct Figure {
    size: Vec3<VoxRel>,
    voxels: Vec<Block>,
}

impl Figure {
    fn calculate_index(&self, off: Vec3<VoxRel>) -> usize {
        (off.x * self.size.y * self.size.z + off.y * self.size.z + off.z) as usize
    }
}

impl Volume for Figure {
    // TODO: Switch this back from Block to Cell since this is a pretty
    // dirty hack while we use a unified volume rendering pipeline.
    type VoxelType = Block;

    fn size(&self) -> Vec3<VoxRel> { self.size }
}

impl ReadVolume for Figure {
    fn at_unchecked(&self, off: Vec3<VoxRel>) -> Self::VoxelType { self.voxels[self.calculate_index(off)] }
}

impl ReadWriteVolume for Figure {
    fn replace_at_unchecked(&mut self, off: Vec3<VoxRel>, vox: Self::VoxelType) -> Self::VoxelType {
        let i = self.calculate_index(off);
        let r = self.voxels[i];
        self.voxels[i] = vox;
        r
    }

    fn fill(&mut self, vox: Self::VoxelType) {
        // Default implementation
        for v in self.voxels.iter_mut() {
            *v = vox;
        }
    }
}

impl ConstructVolume for Figure {
    fn filled(size: Vec3<VoxRel>, vox: Self::VoxelType) -> Figure {
        let vol = Figure {
            size,
            voxels: vec![vox; (size.x * size.y * size.z) as usize],
        };
        vol
    }

    fn empty(size: Vec3<VoxRel>) -> Figure { Self::filled(size, Self::VoxelType::empty()) }
}

impl PhysicalVolume for Figure {
    fn scale(&self) -> Vec3<f32> { Vec3::new(0.085, 0.085, 0.085) }
}

impl Figure {
    pub fn new() -> Self {
        Figure {
            size: Vec3::from((0, 0, 0)),
            voxels: Vec::new(),
        }
    }
}
