// Standard
use std::ops::{Add, Sub, Div, Mul, Neg, Rem};

// Library
use noise::{NoiseFn, SuperSimplex, Seedable, HybridMulti, MultiFractal};
use vek::*;
use dot_vox;

// Project
use common::terrain::chunk::Block;

// Local
use util::structure::{StructureGen, dist_by_euc, dist_by_axis};
use overworldgen::OverworldGen;
use cachegen::CacheGen;
use Gen;
use new_seed;

// <--- BEGIN MESS --->

// Project
use common::terrain::{
    Voxel,
    chunk::HeterogeneousData,
    Volume, ReadVolume, ReadWriteVolume, ConstructVolume,
};

// TODO: Replace this with a superior voxel loading system
// Maybe include_bytes! these files into the executable?
// Might limit modding

fn dot_vox_to_hetero(vox: dot_vox::DotVoxData) -> HeterogeneousData {
    match vox.models.first() {
        Some(model) => {
            let size = Vec3::new(model.size.x, model.size.y, model.size.z).map(|e| e as u32);
            let mut voxels = vec![Block::empty(); (size.x * size.y * size.z) as usize];
            let mut chunk = HeterogeneousData::filled(size, Block::AIR);

            for ref v in model.voxels.iter() {
                let pos = Vec3::new(v.x as u32, v.y as u32, v.z as u32);
                chunk.set_at(pos, match v.i {
                    8 | 9 => Block::AIR,
                    i => Block::from_byte(i),
                });
            }

            chunk
        },
        None => HeterogeneousData::filled(Vec3::zero(), Block::AIR),
    }
}

fn load_trees() -> Vec<HeterogeneousData> {
    let mut trees = vec![];

    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/Veloren_Trees/Birken/Birch_1.vox").unwrap()));
    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/Veloren_Trees/Birken/Birch_2.vox").unwrap()));
    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/Veloren_Trees/Birken/Birch_3.vox").unwrap()));
    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/Veloren_Trees/Birken/Birch_4.vox").unwrap()));
    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/Veloren_Trees/Birken/Birch_5.vox").unwrap()));
    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/Veloren_Trees/Birken/Birch_6.vox").unwrap()));
    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/Veloren_Trees/Birken/Birch_7.vox").unwrap()));
    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/Veloren_Trees/Birken/Birch_8.vox").unwrap()));
    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/Veloren_Trees/Birken/Birch_9.vox").unwrap()));
    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/Veloren_Trees/Birken/Birch_10.vox").unwrap()));

    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/CW_Trees/Pine Trees/A1.vox").unwrap()));
    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/CW_Trees/Pine Trees/A2.vox").unwrap()));
    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/CW_Trees/Pine Trees/B1.vox").unwrap()));
    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/CW_Trees/Pine Trees/B2.vox").unwrap()));
    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/CW_Trees/Pine Trees/PineMK5A.vox").unwrap()));
    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/CW_Trees/Pine Trees/PineMK5A_Snow.vox").unwrap()));

    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/Veloren_Trees/Pappeln/1.vox").unwrap()));
    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/Veloren_Trees/Pappeln/2.vox").unwrap()));
    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/Veloren_Trees/Pappeln/3.vox").unwrap()));
    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/Veloren_Trees/Pappeln/4.vox").unwrap()));
    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/Veloren_Trees/Pappeln/5.vox").unwrap()));
    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/Veloren_Trees/Pappeln/6.vox").unwrap()));
    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/Veloren_Trees/Pappeln/7.vox").unwrap()));
    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/Veloren_Trees/Pappeln/8.vox").unwrap()));
    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/Veloren_Trees/Pappeln/9.vox").unwrap()));
    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/Veloren_Trees/Pappeln/10.vox").unwrap()));

    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/Veloren_Trees/Willows/1.vox").unwrap()));
    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/Veloren_Trees/Willows/2.vox").unwrap()));

    trees
}

lazy_static! {
    static ref TREES: Vec<HeterogeneousData> = load_trees();
}

// <--- END MESS --->

// TODO: Call this file forestgen.rs
type TreeGenOut = Option<(Vec3<i64>, usize)>;

pub struct TreeGen {
    gen: CacheGen<StructureGen, Vec2<i64>, TreeGenOut>,
}

impl TreeGen {
    pub fn new() -> Self {
        Self {
            gen: CacheGen::new(StructureGen::new(
                64, // freq
                48, // warp
                new_seed(), // seed
                dist_by_axis, // distance function
            ), 4096),
        }
    }
}

impl Gen<OverworldGen> for TreeGen {
    type In = Vec3<i64>;
    type Out = Option<Block>;

    fn sample<'a>(&'a self, pos: Vec3<i64>, overworld: &'a OverworldGen) -> Option<Block> {
        if let Some((tree_pos, tree_idx)) = self.gen.sample(Vec2::from(pos), &(overworld, |this: &StructureGen, pos, overworld_gen: &OverworldGen| {
            let overworld = overworld_gen.sample(pos, &());

            if overworld.land > 0.0 && overworld.dry > 0.012 && overworld.dry < 0.4 {
                Some((
                    Vec3::new(pos.x, pos.y, overworld.z_alt as i64 - 1),
                    this.throw_dice(pos, 0) as usize % TREES.len(),
                ))
            } else {
                None
            }
        })) {
            let tree = &TREES[tree_idx];

            let rel_pos = (Vec2::from(pos) - tree_pos) + Vec2::from(tree.size()).map(|e: u32| e as i64) / 2;

            tree.at(Vec3::new(rel_pos.x, rel_pos.y, pos.z - tree_pos.z).map(|e| e as u32))
        } else {
            None
        }
    }
}
