use noise::{NoiseFn, OpenSimplex, Seedable};
use coord::prelude::*;

use {Volume, Voxel, Block, BlockMaterial, BlockKind};

pub struct Chunk {
    size: Vec3<i64>,
    offset: Vec3<i64>,
    voxels: Vec<Block>,
}

impl Chunk {
    pub fn test(offset: Vec3<i64>, size: Vec3<i64>) -> Chunk {

        let offs_x_noise = OpenSimplex::new().set_seed(1);
        let offs_y_noise = OpenSimplex::new().set_seed(2);
        let offs_z_noise = OpenSimplex::new().set_seed(3);

        let height_noise = OpenSimplex::new().set_seed(4);
        let ridge_noise = OpenSimplex::new().set_seed(5);

        let cave_noise_0 = OpenSimplex::new().set_seed(6);
        let cave_noise_1 = OpenSimplex::new().set_seed(7);

        let mountain_noise = OpenSimplex::new().set_seed(8);

        let color_noise = OpenSimplex::new().set_seed(9);

        let terrain_height = 64.0;
        let terrain_scale = 128.0;
        let terrain_turbulence = 24.0;
        let ridge_factor = 0.5;
        let turbulence_scatter = 0.07;
        let mountain_height = 150.0;

        let cave_scale = 64.0;

        let mut voxels = Vec::new();

        for i in 0..size.x {
            for j in 0..size.y {
                for k in 0..size.z {
                    let pos = (vec3!(i, j, k) + offset).map(|e| e as f64);

                    let offs = vec3!(
                        offs_x_noise.get((pos * turbulence_scatter).elements()),
                        offs_y_noise.get((pos * turbulence_scatter).elements()),
                        offs_z_noise.get((pos * turbulence_scatter).elements())
                    ) * terrain_turbulence;

                    let ridge = 1.0 - 2.0 * ridge_noise.get((pos / terrain_scale).elements()).abs();
                    let terrain = height_noise.get(((pos + offs) / terrain_scale).elements()) * (1.0 - ridge_factor) + ridge * ridge_factor;
                    let height = (terrain * mountain_height + terrain_height) as i64;

                    let mountain_offs = (mountain_noise.get([pos.x * 0.05, pos.y * 0.05]) * 32.0) as i64;

                    let cave0 = 1.0 - cave_noise_0.get((pos / cave_scale).elements()).abs();
                    let cave1 = 1.0 - cave_noise_1.get((pos / cave_scale).elements()).abs();

                    let color_var = (color_noise.get(pos.elements()) * 60.0) as u8;

                    voxels.push(Block::new(
                        if k == 0 {
                            BlockMaterial { kind: BlockKind::Stone, color: vec3!(145, 170, 160) + color_var }
                        } else if k <= height {
                            if cave0 + cave1 > 1.94 {
                                BlockMaterial { kind: BlockKind::Air, color: vec3!(0, 0, 0) + color_var }
                            } else if k < height - 4 {
                                BlockMaterial { kind: BlockKind::Stone, color: vec3!(125, 150, 140) + color_var }
                            } else if k < height {
                                BlockMaterial { kind: BlockKind::Earth, color: vec3!(160, 120, 80) + color_var }
                            } else if k <= size.z / 3 + 5 {
                                BlockMaterial { kind: BlockKind::Sand, color: vec3!(225, 205, 100) + color_var }
                            } else if k + mountain_offs > (size.z * 5) / 9 {
                                BlockMaterial { kind: BlockKind::Stone, color: vec3!(145, 170, 160) + color_var }
                            } else {
                                BlockMaterial { kind: BlockKind::Grass, color: vec3!(75, 125, 40) + color_var }
                            }
                        } else {
                            if k <= size.z / 3 {
                                BlockMaterial { kind: BlockKind::Water, color: vec3!(65, 150, 180) + color_var }
                            } else {
                                BlockMaterial { kind: BlockKind::Air, color: vec3!(0, 0, 0) + color_var }
                            }
                        }
                    ));
                }
            }
        }

        Chunk {
            size,
            offset,
            voxels,
        }
    }

    fn pos_to_index(&self, pos: Vec3<i64>) -> usize {
        (pos.x * self.size.y * self.size.z + pos.y * self.size.z + pos.z) as usize
    }
}

impl Volume for Chunk {
    type VoxelType = Block;

    fn new() -> Self {
        Chunk {
            size: Vec3::from((0, 0, 0)),
            offset: Vec3::from((0, 0, 0)),
            voxels: Vec::new(),
        }
    }

    fn fill(&mut self, block: Block) {
        for v in self.voxels.iter_mut() {
            *v = block;
        };
    }

    fn size(&self) -> Vec3<i64> {
        self.size
    }

    fn offset(&self) -> Vec3<i64> {
        self.offset
    }

    fn rotation(&self) -> Vec3<f64> {
        Vec3::new(0.0, 0.0, 0.0)
    }

    fn scale(&self) -> Vec3<f64> {
        Vec3::new(1.0, 1.0, 1.0)
    }

    fn set_size(&mut self, size: Vec3<i64>) {
        self.size = size;
        self.voxels.resize((size.x * size.y * size.z) as usize, Block::empty());
    }

    fn set_offset(&mut self, offset: Vec3<i64>) {
        self.offset = offset;
    }

    fn at(&self, pos: Vec3<i64>) -> Option<Block> {
        if pos.x < 0 || pos.y < 0 || pos.z < 0 ||
            pos.x >= self.size.x || pos.y >= self.size.y || pos.z >= self.size.z
        {
            None
        } else {
            Some(self.voxels[self.pos_to_index(pos)])
        }
    }

    fn set(&mut self, pos: Vec3<i64>, vt: Block) {
        if pos.x < 0 || pos.y < 0 || pos.z < 0 ||
            pos.x >= self.size.x || pos.y >= self.size.y || pos.z >= self.size.z
        {
        } else {
            let i = self.pos_to_index(pos);
            self.voxels[i] = vt;
        }
    }
}
