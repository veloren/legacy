// Standard
use std::ops::{Add, Sub, Div, Mul, Neg, Rem};

// Library
use vek::*;
use noise::{NoiseFn, Seedable, SuperSimplex};
use dot_vox;

// Project
use common::terrain::{
    Voxel,
    chunk::{Block, HeterogeneousData},
    Volume, ReadVolume, ReadWriteVolume, ConstructVolume,
};

// Local
use Gen;
use overworld;

fn dot_vox_to_hetero(vox: dot_vox::DotVoxData) -> HeterogeneousData {
    match vox.models.first() {
        Some(model) => {
            let size = Vec3::new(model.size.x, model.size.y, model.size.z).map(|e| e as u32);
            let mut voxels = vec![Block::empty(); (size.x * size.y * size.z) as usize];
            let mut chunk = HeterogeneousData::filled(size, Block::AIR);

            for ref v in model.voxels.iter() {
                let pos = Vec3::new(v.x as u32, v.y as u32, v.z as u32);
                chunk.set_at(pos, Block::from_byte(v.i));
            }

            chunk
        },
        None => HeterogeneousData::filled(Vec3::zero(), Block::AIR),
    }
}

fn load_trees() -> Vec<HeterogeneousData> {
    let mut trees = vec![];

    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/Pine Trees/A1.vox").unwrap()));
    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/Pine Trees/A2.vox").unwrap()));
    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/Pine Trees/B1.vox").unwrap()));
    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/Pine Trees/B2.vox").unwrap()));

    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/Tree Variations Autumn/Tree12Brown.vox").unwrap()));
    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/Tree Variations Autumn/Tree12Brown2.vox").unwrap()));
    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/Tree Variations Autumn/Tree12Green2.vox").unwrap()));
    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/Tree Variations Autumn/Tree12Green3.vox").unwrap()));
    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/Tree Variations Autumn/Tree12Orange.vox").unwrap()));
    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/Tree Variations Autumn/Tree12Orange2.vox").unwrap()));
    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/Tree Variations Autumn/Tree12Yellow.vox").unwrap()));
    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/Tree Variations Autumn/Tree12yellow2.vox").unwrap()));
    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/Tree Variations Autumn/Tree12Red.vox").unwrap()));
    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/Tree Variations Autumn/Tree12Red2.vox").unwrap()));

    //trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Structures/Human/Houses/16x16x16/Red/5R.vox").unwrap()));
    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Structures/Human/Houses/16x16x16/Turqoise/turq4.vox").unwrap()));
    //trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Structures/Human/Houses/16x16x16/Blue/blue3.vox").unwrap()));

    trees
}

lazy_static! {
    static ref TREES: Vec<HeterogeneousData> = load_trees();
}

#[derive(Copy, Clone)]
pub struct Sample {
    pub block: Option<Block>,
}

pub struct TreeGen {
    turb_nz: (SuperSimplex, SuperSimplex),
}

impl TreeGen {
    pub fn new() -> Self {
        let mut seed = 0;
        let mut new_seed = || { seed += 1; seed };

        Self {
            turb_nz: (
                SuperSimplex::new().set_seed(new_seed()),
                SuperSimplex::new().set_seed(new_seed()),
            ),
        }
    }

    fn get_dice(&self, pos: Vec2<i64>, seed: i64) -> i64 {
        let next = 327387278322 ^ seed * 1103515245 + 15341;
        let next = 327387278322 ^ pos.x * 1103515245 + 12343;
        let next = 327387278322 ^ (next + pos.y) * 1103515245 + 12541;
        next
    }

    fn get_cell_pos(&self, cell_coord: Vec2<i64>, freq: i64, warp: i64) -> Vec2<i64> {
        cell_coord * freq + freq / 2 + Vec2::new(
            self.get_dice(cell_coord, 0).abs(),
            self.get_dice(cell_coord, 1).abs(),
        ).map(|e| e.mod_euc(warp * 2)) - warp
    }

    /// Returns (grid_pos, offset)
    fn get_nearest_tree(&self, pos: Vec3<i64>, overworld: overworld::Sample, alt_surf: f64) -> (Vec2<i64>, Vec3<i64>) {
        let freq = 64;
        let warp = 16;

        let pos2di = Vec2::<i64>::from(pos);

        let cell_coord = pos2di.map(|e| e.div_euc(freq));
        let cell_offs = pos2di.map(|e| e.mod_euc(freq)) - freq / 2;

        let mut min = (cell_coord, freq); // Dummy, to be replaced
        for x in -1..2 {
            for y in -1..2 {
                let dist = (self.get_cell_pos(cell_coord + Vec2::new(x, y), freq, warp) - pos2di).map(|e| e.abs()).reduce_max();
                if dist < min.1 {
                    min = (cell_coord + Vec2::new(x, y), dist);
                }
            }
        }

        let cell_pos = self.get_cell_pos(min.0, freq, warp);

        (
            min.0,
            Vec3::new(cell_pos.x - pos.x, cell_pos.y - pos.y, pos.z - alt_surf as i64 + 2),
        )
    }
}

impl Gen for TreeGen {
    type In = (Vec3<i64>, overworld::Sample, f64);
    type Out = Sample;

    fn sample(&self, i: (Vec3<i64>, overworld::Sample, f64)) -> Sample {
        let pos = i.0;
        let overworld = i.1;
        let alt_surf = i.2;

        let (tree_grid_pos, tree_world_offs) = self.get_nearest_tree(pos, overworld, alt_surf);

        let tree_idx = self.get_dice(tree_grid_pos, 2) as usize % TREES.len();

        let model_offset = tree_world_offs + Vec3::from(Vec2::from(TREES[tree_idx].size()) / 2).map(|e: u32| e as i64);

        let block = if overworld.tree_density > 0.5 {
            TREES[tree_idx].at(model_offset.map(|e| e as u32)).and_then(|b| if b.is_solid() { Some(b) } else { None })
        } else {
            None
        };

        Sample {
            block,
        }
    }
}
