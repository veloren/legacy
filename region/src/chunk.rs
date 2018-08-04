use coord::prelude::*;
use noise::{NoiseFn, OpenSimplex, Seedable};
use rand::{rngs::SmallRng, SeedableRng};

use Block;
use BlockMaterial;
use Volume;
use Voxel;

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
        let ore_noise = OpenSimplex::new().set_seed(13);

        let mountain_noise = OpenSimplex::new().set_seed(8);

        let color_noise = OpenSimplex::new().set_seed(9);

        let temp_noise = OpenSimplex::new().set_seed(10);

        let terrain_height = 64.0;
        let terrain_scale = 128.0;
        let terrain_turbulence = 24.0;
        let ridge_factor = 0.5;
        let turbulence_scatter = 0.07;
        let mountain_height = 220.0;
        let biome_scale = 1024.0;
        let forest_scale = 512.0;

        let cave_scale = 64.0;
        let ore_scarcity = 48.0;

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
                    let terrain = height_noise.get(((pos + offs) / terrain_scale).elements()) * (1.0 - ridge_factor)
                        + ridge * ridge_factor;
                    let height = (terrain * mountain_height + terrain_height) as i64;

                    voxels.push(Block::new(if k == 0 {
                        BlockMaterial::Stone
                    } else if k <= height {
                        let cave0 = 1.0 - cave_noise_0.get((pos / cave_scale).elements()).abs();
                        let cave1 = 1.0 - cave_noise_1.get((pos / cave_scale).elements()).abs();

                        if cave0 + cave1 > 1.94 {
                            BlockMaterial::Air
                        } else if k < height - 4 {
                            if ore_noise.get((pos / ore_scarcity).elements()) > 0.4 {
                                BlockMaterial::Gold
                            } else {
                                BlockMaterial::Stone
                            }
                        } else if k < height {
                            BlockMaterial::Earth
                        } else if k <= size.z / 3 + 5 {
                            BlockMaterial::Sand
                        } else {
                            BlockMaterial::Earth
                        }
                    } else {
                        if k <= size.z / 3 {
                            BlockMaterial::Water
                        } else {
                            BlockMaterial::Air
                        }
                    }));
                }
            }
        }

        let mut chunk = Chunk { size, offset, voxels };

        let tree_noise = OpenSimplex::new().set_seed(11);
        let forest_noise = OpenSimplex::new().set_seed(12);

        let boulder_noise = OpenSimplex::new().set_seed(14);

        for i in 0..size.x {
            for j in 0..size.y {
                let pos2d = (vec2!(i, j) + vec2!(offset.x, offset.y)).map(|e| e as f64);

                let offs2d = vec2!(
                    offs_x_noise.get((pos2d * 0.3).elements()),
                    offs_y_noise.get((pos2d * 0.3).elements())
                ) * 32.0;

                let mountain_offs = (mountain_noise.get([pos2d.x * 0.05, pos2d.y * 0.05]) * 32.0) as i64;

                let temp = temp_noise.get(((pos2d + offs2d) / biome_scale).elements());

                let forest = forest_noise.get(((pos2d + offs2d) / forest_scale).elements()) * 0.4;

                for k in 0..size.z {
                    if chunk
                        .at(vec3!(i, j, k))
                        .unwrap_or(Block::new(BlockMaterial::Air))
                        .material()
                        == BlockMaterial::Earth
                        && chunk
                            .at(vec3!(i, j, k + 1))
                            .unwrap_or(Block::new(BlockMaterial::Air))
                            .material()
                            == BlockMaterial::Air
                    {
                        if boulder_noise.get((pos2d * 123.573).elements()) > 0.54 {
                            for ii in -4..5 {
                                for jj in -4..5 {
                                    for kk in -4..5 {
                                        if ii * ii + jj * jj + kk * kk < 25 {
                                            chunk.set(vec3!(i + ii, j + jj, k + kk), Block::new(BlockMaterial::Stone));
                                        }
                                    }
                                }
                            }
                        } else if tree_noise.get((pos2d * 10.0).elements()) < forest - 0.6 {
                            let rng =
                                SmallRng::from_seed([i as u8, j as u8, k as u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
                            for branch in 10..40 {
                                let v = vec2!(
                                    tree_noise.get((pos2d * 100.0 + branch as f64 + 0.0).elements()),
                                    tree_noise.get((pos2d * 100.0 + branch as f64 + 100.0).elements())
                                ).norm();
                                for l in 0..25 {
                                    let inc = v.map(|e| (e * (1.0 - 0.025 * branch as f64) * 0.5 * l as f64) as i64);
                                    chunk.set(
                                        vec3!(i + inc.x, j + inc.y, k + branch / 2),
                                        Block::new(BlockMaterial::Leaves),
                                    );
                                }
                            }

                            for trunk in 0..10 {
                                chunk.set(vec3!(i, j, k + trunk), Block::new(BlockMaterial::Log));
                            }
                        } else {
                            chunk.set(
                                vec3!(i, j, k),
                                Block::new(if k + mountain_offs > (size.z * 7) / 9 {
                                    BlockMaterial::Stone
                                } else if temp < -0.2 {
                                    BlockMaterial::Snow
                                } else if temp > 0.2 {
                                    BlockMaterial::Sand
                                } else {
                                    BlockMaterial::Grass
                                }),
                            );
                        }
                    }
                }
            }
        }

        chunk
    }

    fn pos_to_index(&self, pos: Vec3<i64>) -> usize {
        (pos.x * self.size.y * self.size.z + pos.y * self.size.z + pos.z) as usize
    }

    pub fn new() -> Self {
        Chunk {
            size: Vec3::from((0, 0, 0)),
            offset: Vec3::from((0, 0, 0)),
            voxels: Vec::new(),
        }
    }
}

impl Volume for Chunk {
    type VoxelType = Block;

    fn fill(&mut self, block: Block) {
        for v in self.voxels.iter_mut() {
            *v = block;
        }
    }

    fn size(&self) -> Vec3<i64> { self.size }

    fn offset(&self) -> Vec3<i64> { self.offset }

    fn ori(&self) -> Vec3<f32> { Vec3::new(0.0, 0.0, 0.0) }

    fn scale(&self) -> Vec3<f32> { Vec3::new(1.0, 1.0, 1.0) }

    fn set_size(&mut self, size: Vec3<i64>) {
        self.size = size;
        self.voxels.resize((size.x * size.y * size.z) as usize, Block::empty());
    }

    fn set_offset(&mut self, offset: Vec3<i64>) { self.offset = offset; }

    fn at(&self, pos: Vec3<i64>) -> Option<Block> {
        if pos.x < 0 || pos.y < 0 || pos.z < 0 || pos.x >= self.size.x || pos.y >= self.size.y || pos.z >= self.size.z {
            None
        } else {
            Some(self.voxels[self.pos_to_index(pos)])
        }
    }

    fn set(&mut self, pos: Vec3<i64>, vt: Block) {
        if pos.x < 0 || pos.y < 0 || pos.z < 0 || pos.x >= self.size.x || pos.y >= self.size.y || pos.z >= self.size.z {
        } else {
            let i = self.pos_to_index(pos);
            self.voxels[i] = vt;
        }
    }
}
