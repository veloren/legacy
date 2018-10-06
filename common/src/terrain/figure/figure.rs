// Standard
use std::any::Any;

// Library
use vek::*;

// Local
use terrain::{
    figure::{Cell, CellMaterial},
    Volume, Voxel, ReadVolume, ReadWriteVolume, ConstructVolume, VoxelRelVec,
};

pub struct Figure {
    size: VoxelRelVec,
    voxels: Vec<Cell>,
}

impl Figure {
    pub fn test(offset: Vec3<i64>, size: VoxelRelVec) -> Figure {
        let mut voxels = Vec::new();

        for _i in 0..size.x {
            for _j in 0..size.y {
                for _k in 0..size.z {
                    voxels.push(Cell::new(CellMaterial::MatteSmooth(0)));
                }
            }
        }

        Figure {
            size,
            voxels,
        }
    }

    fn calculate_index(&self, off: VoxelRelVec) -> usize {
        (off.x * self.size.y * self.size.z + off.y * self.size.z + off.z) as usize
    }
}

impl Volume for Figure {
    type VoxelType = Cell;

    fn size(&self) -> VoxelRelVec { self.size }
}

impl ReadVolume for Figure {
    fn at_unsafe(&self, off: VoxelRelVec) -> Cell {
        self.voxels[self.calculate_index(off)]
    }
}

impl ReadWriteVolume for Figure {
    fn replace_at_unsafe(&mut self, off: VoxelRelVec, vox: Self::VoxelType) -> Self::VoxelType {
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
    fn filled(size: VoxelRelVec, vox: Self::VoxelType) -> Figure {
        let mut vol = Figure {
            size,
            voxels: vec![vox; (size.x * size.y * size.z) as usize],
        };
        vol
    }

    fn empty(size: VoxelRelVec) -> Figure {
        Self::filled(size, Cell::empty())
    }
}

/*
impl Volume for Figure {
    type VoxelType = Cell;

    fn fill(&mut self, cell: Cell) {
        for v in self.voxels.iter_mut() {
            *v = cell;
        }
    }

    fn size(&self) -> Vec3<i64> { self.size }

    fn offset(&self) -> Vec3<i64> { self.offset }

    fn ori(&self) -> Vec3<f32> { self.ori }

    fn scale(&self) -> Vec3<f32> { self.scale }

    fn set_size(&mut self, size: Vec3<i64>) {
        self.size = size;
        self.voxels.resize(
            (size.x * size.y * size.z) as usize,
            Cell::new(CellMaterial::MatteSmooth(0)),
        );
    }

    fn set_offset(&mut self, offset: Vec3<i64>) { self.offset = offset; }

    fn at(&self, pos: Vec3<i64>) -> Option<Cell> {
        if pos.x < 0 || pos.y < 0 || pos.z < 0 || pos.x >= self.size.x || pos.y >= self.size.y || pos.z >= self.size.z {
            None
        } else {
            Some(self.voxels[self.pos_to_index(pos)])
        }
    }

    fn set(&mut self, pos: Vec3<i64>, vt: Cell) {
        if pos.x < 0 || pos.y < 0 || pos.z < 0 || pos.x >= self.size.x || pos.y >= self.size.y || pos.z >= self.size.z {
        } else {
            let i = self.pos_to_index(pos);
            self.voxels[i] = vt;
        }
    }

    fn as_any_mut(&mut self) -> &mut Any { self }

    fn as_any(&self) -> &Any { self }
}*/

impl Figure {
    pub fn new() -> Self {
        Figure {
            size: Vec3::from((0, 0, 0)),
            voxels: Vec::new(),
        }
    }
}
