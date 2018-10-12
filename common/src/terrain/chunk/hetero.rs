// Library
use noise::{NoiseFn, SuperSimplex, HybridMulti, Seedable, MultiFractal};
use vek::*;
use dot_vox::DotVoxData;

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

impl From<DotVoxData> for Chunk {
    fn from(vox: DotVoxData) -> Chunk {
        match vox.models.first() {
            Some(model) => {
                let size = Vec3::new(model.size.x, model.size.y, model.size.z).map(|e| e as i64);
                let mut voxels = vec![Block::empty(); (size.x * size.y * size.z) as usize];
                let mut chunk = Chunk {
                    size,
                    offset: Vec3::new(0, 0, 0),
                    voxels,
                };

                for ref v in model.voxels.iter() {
                    let pos = Vec3::new(v.x as i64, v.y as i64, v.z as i64);
                    chunk.set(pos, Block::from_byte(v.i));
                }

                chunk
            },
            None => Chunk {
                size: Vec3::new(1, 1, 1),
                offset: Vec3::new(0, 0, 0),
                voxels: vec![Block::empty()]
            },
        }
    }
}

impl Chunk {
    pub fn test(offset: Vec3<i64>, size: Vec3<i64>) -> Chunk {
        let mut voxels = Vec::new();

        let mut seed = 0;
        let mut new_seed = || { seed += 1; seed };

        let dry_nz = HybridMulti::new()
            .set_seed(new_seed()).set_octaves(4);
        let chaos_nz = SuperSimplex::new()
            .set_seed(new_seed());
        let cliff_nzs = (
            SuperSimplex::new().set_seed(new_seed()),
            SuperSimplex::new().set_seed(new_seed()),
        );
        let cliff_vari_nz = SuperSimplex::new()
            .set_seed(new_seed());
        let hill_nz = SuperSimplex::new()
            .set_seed(new_seed());
        let ridge_nz = HybridMulti::new()
            .set_seed(new_seed());
        let peak_nz = HybridMulti::new()
            .set_seed(new_seed())
            .set_octaves(2);
        let house_nzs = (
            SuperSimplex::new().set_seed(new_seed()),
            SuperSimplex::new().set_seed(new_seed()),
        );
        let tree_nzs = (
            SuperSimplex::new().set_seed(new_seed()),
            SuperSimplex::new().set_seed(new_seed()),
        );

        //let tree_vol = Chunk::from(dot_vox::load("../assets/pending/struc32/strucsub321.vox")
        //    .expect("cannot find house"));

        for i in 0..size.x {
            for j in 0..size.y {
                let pos2d = (Vec3::new(i + offset.x, j + offset.y, 0)).map(|e| e as f64);

                let dry = { // 0.0 = wet, 1.0 = dry
                    let scale = 2048.0;

                    dry_nz.get(pos2d.div(scale).into_array()).mul(1.5).abs().min(1.0)
                };

                let chaos = { // 0.0 = normal/low, 1.0 = high
                    let scale = 1024.0;

                    chaos_nz.get(pos2d.div(scale).into_array()).mul(dry).powf(2.0).mul(4.0).max(0.0).min(1.0)
                };

                let river = { // 0.0 = normal/flat, max_depth = deepest
                    let depth = 24.0;
                    let max_depth = 16.0;

                    dry.neg().add(1.0).powf(16.0).mul(depth).min(max_depth)
                };

                let hill = { // 0.0 = normal/flat, 1.0 = highest
                    let scale = 512.0;
                    let amp = 16.0;

                    hill_nz.get(pos2d.div(scale).into_array()).abs().mul(dry).mul(amp)
                };

                let ridge = {
                    let scale = 1024.0;
                    let height = 130.0;

                    (1.0 - peak_nz.get(pos2d.div(scale).into_array()).abs()).mul(chaos).mul(height)
                };

                let house = {
                    let scale = 150.0;

                    let mut min_dist = std::f64::MAX;
                    let mut min_diff = Vec3::new(0.0, 0.0, 0.0);
                    for i in 0..3 {
                        for j in 0..3 {
                            let cell_pos = pos2d.div(scale).add(Vec3::new(i as f64, j as f64, 0.0)).map(|e| e.floor());
                            let point_pos = cell_pos + Vec3::new(
                                house_nzs.0.get(cell_pos.mul(10.0).into_array()) * 0.8,
                                house_nzs.1.get(cell_pos.mul(10.0).into_array()) * 0.8,
                                0.0
                            );
                            let samp_pos = pos2d.div(scale);
                            let diff = samp_pos - point_pos;
                            let dist = diff.x.abs().max(diff.y.abs());
                            if dist < min_dist {
                                min_dist = dist;
                                min_diff = diff;
                            }
                        }
                    }

                    struct House {
                        is_wall: bool,
                        is_roof: bool,
                        roof_height: f64,
                        is_door: bool,
                        is_window: bool,
                    }

                    let is_wall = 0.1 < min_dist && min_dist < 0.11;

                    House {
                        is_wall,
                        is_roof: min_dist < 0.13,
                        roof_height: 16.0 * (1.0 - min_dist * 10.0),
                        is_door: min_diff.x.abs() < 0.015 && min_diff.y > 0.0,
                        is_window: (min_diff.x.abs().rem(0.1) > 0.05 || min_diff.y.abs().rem(0.1) > 0.05) && is_wall,
                    }
                };

                let tree_off = {
                    let scale = 150.0;

                    let mut min_dist = std::f64::MAX;
                    let mut min_diff = Vec3::new(0.0, 0.0, 0.0);
                    for i in 0..3 {
                        for j in 0..3 {
                            let cell_pos = pos2d.div(scale).add(Vec3::new(i as f64, j as f64, 0.0)).map(|e| e.floor());
                            let point_pos = cell_pos + Vec3::new(
                                tree_nzs.0.get(cell_pos.mul(10.0).into_array()) * 0.8,
                                tree_nzs.1.get(cell_pos.mul(10.0).into_array()) * 0.8,
                                0.0
                            );
                            let samp_pos = pos2d.div(scale);
                            let diff = samp_pos - point_pos;
                            let dist = diff.x.abs().max(diff.y.abs());
                            if dist < min_dist {
                                min_dist = dist;
                                min_diff = diff;
                            }
                        }
                    }

                    min_diff.map(|e| (e * scale) as i64)
                };

                for k in 0..size.z {
                    let base_surf = 64.0;
                    let water_level = base_surf - 6.0;

                    let pos3d = (Vec3::new(i, j, k) + offset).map(|e| e as f64);

                    let cliff_height = {
                        let scale = 64.0;
                        let vari = 0.3;
                        let height = 192.0;

                        cliff_vari_nz.get(pos3d.div(scale).into_array()).mul(vari).add(1.0).mul(height)
                    };

                    let cliff = {
                        let spot_scales_xy = (128.0, 64.0);
                        let spot_scales_z = (256.0, 128.0);
                        let spot_scales = (
                            Vec3::new(spot_scales_xy.0, spot_scales_xy.0, spot_scales_z.0),
                            Vec3::new(spot_scales_xy.1, spot_scales_xy.1, spot_scales_z.1),
                        );
                        let layers = 4.0;

                        let spots = (
                            cliff_nzs.0.get(pos3d.div(spot_scales.0).into_array()),
                            cliff_nzs.1.get(pos3d.div(spot_scales.1).into_array()),
                        );
                        (
                            chaos.mul(0.3) +
                            spots.0.mul(chaos).mul(dry).mul(0.5) +
                            spots.1.mul(dry).mul(0.1)
                        ).mul(layers).round().div(layers).max(0.0).mul(cliff_height)
                    };

                    let peak = {
                        let scale = 128.0;
                        let height = 30.0;

                        peak_nz.get(pos3d.div(scale).into_array()).mul(chaos).mul(height)
                    };

                    let basic_surf = base_surf + hill;
                    let alt_surf = basic_surf - river + cliff.max(peak + ridge);

                    let surf = alt_surf;

                    voxels.push(
                        if pos3d.z < surf {
                            if pos3d.z < water_level {
                                Block::new(BlockMaterial::Sand)
                            } else if pos3d.z < surf - 8.0 {
                                Block::new(BlockMaterial::Stone)
                            } else if pos3d.z < surf - 3.0 {
                                Block::new(BlockMaterial::Earth)
                            } else {
                                Block::new(BlockMaterial::Grass)
                            }
                        } else {
                            if house.is_roof && pos3d.z < alt_surf + 1.0 {
                                Block::new(BlockMaterial::Stone)
                            } else if house.is_roof && pos3d.z > alt_surf + 16.0 && pos3d.z < alt_surf + 22.0 + house.roof_height {
                                Block::new(BlockMaterial::Stone)
                            } else if house.is_window && pos3d.z < alt_surf + 12.0 && pos3d.z > alt_surf + 5.0 && !house.is_door {
                                Block::new(BlockMaterial::Water)
                            } else if house.is_wall && pos3d.z < alt_surf + 20.0 && !(house.is_door && pos3d.z < alt_surf + 5.0) {
                                Block::new(BlockMaterial::Log)
                            } else if pos3d.z < water_level {
                                Block::new(BlockMaterial::Water)
                            } else {
                                Block::new(BlockMaterial::Air)
                            }
                            /*
                            match tree_vol.at(tree_off + Vec3::new(0, 0, -6 + pos3d.z as i64 - basic_surf as i64) + tree_vol.size() / 2).map(|b| b.material()).unwrap_or(BlockMaterial::Air) {
                                BlockMaterial::Air => {
                                    if pos3d.z < water_level {
                                        Block::new(BlockMaterial::Water)
                                    } else {
                                        Block::new(BlockMaterial::Air)
                                    }
                                },
                                m => Block::new(m),
                            }
                            */
                        }
                    );
                }
            }
        }

        Chunk { size, offset, voxels }
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
