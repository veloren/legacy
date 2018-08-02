use coord::prelude::*;

use Cell;
use Volume;
use Voxel;

pub struct Figure {
    size: Vec3<i64>,
    offset: Vec3<i64>,
    ori: Vec3<f32>,
    scale: Vec3<f32>,
    voxels: Vec<Cell>,
}

impl Figure {
    pub fn test(offset: Vec3<i64>, size: Vec3<i64>) -> Figure {
        let mut voxels = Vec::new();

        for _i in 0..size.x {
            for _j in 0..size.y {
                for _k in 0..size.z {
                    voxels.push(Cell::new(0));
                }
            }
        }

        Figure {
            size,
            offset,
            voxels,
            ori: Vec3::new(0.0, 0.0, 0.0),
            scale: Vec3::new(1.0, 1.0, 1.0),
        }
    }

    fn pos_to_index(&self, pos: Vec3<i64>) -> usize {
        (pos.x * self.size.y * self.size.z + pos.y * self.size.z + pos.z) as usize
    }

    pub fn set_ori(&mut self, ori: Vec3<f32>) { self.ori = ori; }

    pub fn set_scale(&mut self, scale: Vec3<f32>) { self.scale = scale; }
}

impl Volume for Figure {
    type VoxelType = Cell;

    fn new() -> Self {
        Figure {
            size: Vec3::from((0, 0, 0)),
            offset: Vec3::from((0, 0, 0)),
            voxels: Vec::new(),
            ori: Vec3::new(0.0, 0.0, 0.0),
            scale: Vec3::new(1.0, 1.0, 1.0),
        }
    }

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
        self.voxels.resize((size.x * size.y * size.z) as usize, Cell::new(0));
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
}
