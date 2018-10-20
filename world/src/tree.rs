// Standard
use std::ops::{Add, Sub, Div, Mul, Neg, Rem};

// Library
use vek::*;
use noise::{NoiseFn, Seedable, SuperSimplex};
use dot_vox;

// Project
use common::terrain::{
    Volume, Voxel,
    chunk::{Block, Chunk},
};

// Local
use Gen;
use overworld;

fn load_trees() -> Vec<Chunk> {
    let mut trees = vec![];

    trees.push(Chunk::from(dot_vox::load("../assets/world/Trees/Pine Trees/B2.vox")
        .expect("cannot find tree")));

    trees
}

lazy_static! {
    static ref TREES: Vec<Chunk> = load_trees();
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

    /// Returns (grid_pos, offset)
    fn get_nearest_tree(&self, pos: Vec3<i64>, overworld: overworld::Sample, basic_surf: f64) -> (Vec2<i64>, Vec3<i64>) {
        let freq = 64;

        let pos2di = Vec2::<i64>::from(pos);

        let tree_grid_pos = pos2di.map(|e| e.div_euc(freq));
        let tree_world_offs = pos2di.map(|e| e.mod_euc(freq)) - freq / 2;

        let seed = tree_grid_pos.x * 1103515245 + 12345;
        let seed = (seed + tree_grid_pos.y) * 1103515245 + 12345;
        let offsx = seed % (freq / 3) - freq / 6;
        let seed = seed * 1103515245 + 12345;
        let offsy = seed % (freq / 3) - freq / 6;

        (
            tree_grid_pos,
            Vec3::new(tree_world_offs.x + offsx, tree_world_offs.y + offsy, pos.z - basic_surf as i64),
        )
    }
}

impl Gen for TreeGen {
    type In = (Vec3<i64>, overworld::Sample, f64);
    type Out = Sample;

    fn sample(&self, i: (Vec3<i64>, overworld::Sample, f64)) -> Sample {
        let freq = 64;

        let pos = i.0;
        let overworld = i.1;
        let basic_surf = i.2;

        let (tree_grid_pos, tree_world_offs) = self.get_nearest_tree(pos, overworld, basic_surf);

        let model_offset = tree_world_offs + Vec3::from(Vec2::from(TREES[0].size())) / 2;

        Sample {
            block: TREES[0].at(model_offset).and_then(|b| if b.is_solid() { Some(b) } else { None })
        }
    }
}
