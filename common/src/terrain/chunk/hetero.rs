// Library
use noise::{NoiseFn, OpenSimplex, Seedable};
use rand::{prng::XorShiftRng, RngCore, SeedableRng};
use vek::*;

// Local
use terrain::{
    chunk::{Block, BlockMaterial},
    ConstructVolume, PhysicalVolume, ReadVolume, ReadWriteVolume, Volume, VoxAbs, VoxRel, Voxel,
};

#[derive(Clone, Debug, PartialEq)]
pub struct HeterogeneousData {
    size: Vec3<VoxRel>,
    voxels: Vec<Block>,
}

impl HeterogeneousData {
    pub fn test(offset: Vec3<VoxAbs>, size: Vec3<VoxRel>) -> HeterogeneousData {
        let offs_x_noise = OpenSimplex::new().set_seed(1);
        let offs_y_noise = OpenSimplex::new().set_seed(2);
        let offs_z_noise = OpenSimplex::new().set_seed(3);

        let height_noise = OpenSimplex::new().set_seed(4);
        let ridge_noise = OpenSimplex::new().set_seed(5);

        let cave_noise_0 = OpenSimplex::new().set_seed(6);
        let cave_noise_1 = OpenSimplex::new().set_seed(7);
        let ore_noise = OpenSimplex::new().set_seed(13);
        let chaos_noise = OpenSimplex::new().set_seed(14);
        let continent_noise = OpenSimplex::new().set_seed(15);

        let mountain_noise = OpenSimplex::new().set_seed(8);

        let _color_noise = OpenSimplex::new().set_seed(9);

        let temp_noise = OpenSimplex::new().set_seed(10);

        let terrain_height = 85.0;
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
                    let pos = (Vec3::new(i, j, k).map(|e| e as i64) + offset).map(|e| e as f64);

                    let chaos = chaos_noise.get((pos / 256.0).into_array()).abs() * 3.0;

                    let offs = Vec3::new(
                        offs_x_noise.get((pos * turbulence_scatter).into_array()),
                        offs_y_noise.get((pos * turbulence_scatter).into_array()),
                        offs_z_noise.get((pos * turbulence_scatter).into_array()),
                    ) * terrain_turbulence;

                    let ridge = 1.0 - 2.0 * ridge_noise.get((pos / terrain_scale).into_array()).abs();
                    let terrain = height_noise.get(((pos + offs) / terrain_scale).into_array()) * (1.0 - ridge_factor)
                        + ridge * ridge_factor * chaos;

                    let continent = continent_noise.get((pos / 1024.0).into_array()) * 32.0;
                    let height = (terrain * mountain_height * chaos + terrain_height + continent) as f64;

                    voxels.push(Block::new(if pos.z == 0.0 {
                        BlockMaterial::Stone
                    } else if pos.z <= height {
                        let cave0 = 1.0 - cave_noise_0.get((pos / cave_scale).into_array()).abs();
                        let cave1 = 1.0 - cave_noise_1.get((pos / cave_scale).into_array()).abs();

                        if cave0 * cave1 + cave0 + cave1 > 2.85 {
                            BlockMaterial::Air
                        } else if pos.z < height - 4.0 {
                            if ore_noise.get((pos / ore_scarcity).into_array()) > 0.4 {
                                BlockMaterial::Gold
                            } else {
                                BlockMaterial::Stone
                            }
                        } else if pos.z < height {
                            BlockMaterial::Earth
                        } else if pos.z <= (size.z as f64) / 3.0 + 5.0 {
                            BlockMaterial::Sand
                        } else {
                            BlockMaterial::Earth
                        }
                    } else {
                        if pos.z <= (size.z as f64) / 3.0 {
                            BlockMaterial::Water
                        } else {
                            BlockMaterial::Air
                        }
                    }));
                }
            }
        }

        let mut chunk = HeterogeneousData { size, voxels };

        let tree_noise = OpenSimplex::new().set_seed(11);
        let forest_noise = OpenSimplex::new().set_seed(12);

        let boulder_noise = OpenSimplex::new().set_seed(14);

        for i in 0..size.x {
            for j in 0..size.y {
                let pos2d = (Vec2::new(i, j).map(|e| e as i64) + Vec2::new(offset.x, offset.y)).map(|e| e as f64);

                let offs2d = Vec2::new(
                    offs_x_noise.get((pos2d * 0.3).into_array()),
                    offs_y_noise.get((pos2d * 0.3).into_array()),
                ) * 32.0;

                let mountain_offs = (mountain_noise.get([pos2d.x * 0.05, pos2d.y * 0.05]) * 32.0) as i64;

                let temp = temp_noise.get(((pos2d + offs2d) / biome_scale).into_array());

                let forest = forest_noise.get(((pos2d + offs2d) / forest_scale).into_array()) * 0.2;

                for k in 0..size.z {
                    if chunk.at_unchecked(Vec3::new(i, j, k)).material() == BlockMaterial::Earth
                        && k < size.z - 1
                        && chunk.at_unchecked(Vec3::new(i, j, k + 1)).material() == BlockMaterial::Air
                    {
                        if boulder_noise.get((pos2d * 123.573).into_array()) > 0.54 {
                            let mut rng = XorShiftRng::from_seed([
                                i as u8, j as u8, k as u8, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                            ]);
                            for ii in -4..5 {
                                for jj in -4..5 {
                                    for kk in -4..5 {
                                        if ii * ii + jj * jj + kk * kk < 25 + rng.next_u32() as i64 % 5 {
                                            let off = Vec3::new(i as i64 + ii, j as i64 + jj, k as i64 + kk);
                                            chunk.set_at(off.map(|e| e as u16), Block::new(BlockMaterial::Stone));
                                        }
                                    }
                                }
                            }
                        } else if tree_noise.get((pos2d * 10.0).into_array()) < forest - 0.56 {
                            let mut rng = XorShiftRng::from_seed([
                                i as u8, j as u8, k as u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                            ]);

                            let big = rng.next_u32() as i64 % 4;

                            for branch in 6 + big..40 + big * 2 {
                                let v = Vec2::new(
                                    tree_noise.get((pos2d * 100.0 + branch as f64 + 0.0).into_array()),
                                    tree_noise.get((pos2d * 100.0 + branch as f64 + 100.0).into_array()),
                                )
                                .normalized();
                                for l in 0..25 + big * 4 {
                                    let inc = v.map(|e| (e * (1.0 - 0.025 * branch as f64) * 0.5 * l as f64) as i64);
                                    let off = Vec3::new(i as i64 + inc.x, j as i64 + inc.y, k as i64 + branch / 2);
                                    chunk.set_at(off.map(|e| e as u16), Block::new(BlockMaterial::Leaves));
                                }
                            }

                            for trunk in 0..6 + big as u16 {
                                let off = Vec3::new(i, j, k + trunk);
                                chunk.set_at(off, Block::new(BlockMaterial::Log));
                            }
                        } else {
                            let off = Vec3::new(i, j, k);
                            chunk.set_at(
                                off,
                                Block::new(if k as i64 + mountain_offs > (size.z as i64 * 7) / 9 {
                                    BlockMaterial::Stone
                                } else if k < size.z / 3 + 3 {
                                    BlockMaterial::Sand
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
