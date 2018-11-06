// Library
use vek::*;

// Local
use terrain::{
    figure::{Cell, CellMaterial},
    ConstructVolume, PhysicalVolume, ReadVolume, ReadWriteVolume, Volume, VoxRel, Voxel,
};

pub struct Figure {
    size: Vec3<VoxRel>,
    voxels: Vec<Cell>,
}

impl Figure {
    pub fn test(offset: Vec3<i64>, size: Vec3<VoxRel>) -> Figure {
        let mut voxels = Vec::new();

        for _i in 0..size.x {
            for _j in 0..size.y {
                for _k in 0..size.z {
                    voxels.push(Cell::new(CellMaterial::MatteSmooth(0)));
                }
            }
        }

        Figure { size, voxels }
    }

    fn calculate_index(&self, off: Vec3<VoxRel>) -> usize {
        (off.x * self.size.y * self.size.z + off.y * self.size.z + off.z) as usize
    }
}

impl Volume for Figure {
    type VoxelType = Cell;

    fn size(&self) -> Vec3<VoxRel> { self.size }
}

impl ReadVolume for Figure {
    fn at_unchecked(&self, off: Vec3<VoxRel>) -> Cell { self.voxels[self.calculate_index(off)] }
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

    fn empty(size: Vec3<VoxRel>) -> Figure { Self::filled(size, Cell::empty()) }
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
