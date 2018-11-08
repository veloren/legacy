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
use overworldgen::{OverworldGen, Out as OverworldOut};
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
                    7...16 | 224...255 => Block::AIR,
                    i => Block::from_byte(i),
                });
            }

            chunk
        },
        None => HeterogeneousData::filled(Vec3::zero(), Block::AIR),
    }
}

fn load_buildings() -> Vec<HeterogeneousData> {
    let mut buildings = vec![];

    buildings.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Structures/Human/Houses/16x16x16/Turqoise/turq1.vox").unwrap()));
    buildings.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Structures/Human/Houses/16x16x16/Turqoise/turq2.vox").unwrap()));
    buildings.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Structures/Human/Houses/16x16x16/Turqoise/turq3.vox").unwrap()));
    //buildings.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Structures/Human/Houses/16x16x16/Turqoise/turq4.vox").unwrap()));
    //buildings.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Structures/Human/Houses/16x16x16/Turqoise/turq5.vox").unwrap()));
    buildings.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Structures/Human/Houses/16x16x16/Blue/blue1.vox").unwrap()));
    buildings.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Structures/Human/Houses/16x16x16/Blue/blue2.vox").unwrap()));
    buildings.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Structures/Human/Houses/16x16x16/Blue/blue3.vox").unwrap()));
    //buildings.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Structures/Human/Houses/16x16x16/Blue/blue4.vox").unwrap()));
    //buildings.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Structures/Human/Houses/16x16x16/Blue/blue5.vox").unwrap()));
    buildings.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Structures/Human/Houses/16x16x16/Red/1R.vox").unwrap()));
    buildings.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Structures/Human/Houses/16x16x16/Red/2R.vox").unwrap()));
    buildings.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Structures/Human/Houses/16x16x16/Red/3R.vox").unwrap()));
    //buildings.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Structures/Human/Houses/16x16x16/Red/4R.vox").unwrap()));
    //buildings.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Structures/Human/Houses/16x16x16/Red/5R.vox").unwrap()));
    buildings.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Structures/Human/Houses/16x16x16/Green/green1.vox").unwrap()));
    buildings.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Structures/Human/Houses/16x16x16/Green/green2.vox").unwrap()));
    //buildings.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Structures/Human/Houses/16x16x16/Green/green3.vox").unwrap()));
    //buildings.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Structures/Human/Houses/16x16x16/Green/green4.vox").unwrap()));
    //buildings.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Structures/Human/Houses/16x16x16/Green/green5.vox").unwrap()));

    //buildings.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Structures/Human/townhall.vox").unwrap()));
    //buildings.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Structures/Human/tower.vox").unwrap()));

    buildings.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/Veloren_Trees/Birken/Birch_1.vox").unwrap()));
    buildings.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/Veloren_Trees/Pappeln/1.vox").unwrap()));
    //buildings.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/Veloren_Trees/Willows/1.vox").unwrap()));

    buildings
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

    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/Veloren_Trees/Oaks/Oak1.vox").unwrap()));
    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/Veloren_Trees/Oaks/Oak2.vox").unwrap()));
    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/Veloren_Trees/Oaks/Oak3.vox").unwrap()));
    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/Veloren_Trees/Oaks/Oak4.vox").unwrap()));
    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/Veloren_Trees/Oaks/Oak5.vox").unwrap()));
    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/Veloren_Trees/Oaks/Oak6.vox").unwrap()));
    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/Veloren_Trees/Oaks/Oak7.vox").unwrap()));
    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/Veloren_Trees/Oaks/Oak8.vox").unwrap()));
    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/Veloren_Trees/Oaks/Oak9.vox").unwrap()));
    trees.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/Veloren_Trees/Oaks/Oak10.vox").unwrap()));

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
    static ref BUILDINGS: Vec<HeterogeneousData> = load_buildings();
    static ref TREES: Vec<HeterogeneousData> = load_trees();
}

// <--- END MESS --->

#[derive(Copy, Clone)]
pub struct Out {
    pub is_town: bool,
    pub surface: Option<Block>,
    pub block: Option<Block>,
}

#[derive(Copy, Clone)]
enum CityResult {
    Town,
    Pyramid { height: u64, z: i64 },
    None,
}

#[derive(Copy, Clone)]
enum BuildingResult {
    House { idx: usize, unit_x: Vec2<i64>, unit_y: Vec2<i64> },
    Park,
    Tree { idx: usize, unit_x: Vec2<i64>, unit_y: Vec2<i64> },
    Rock,
    Pyramid { height: u64 },
    None,
}

type CityGenOut = (Vec2<i64>, CityResult);
type BuildingGenOut = (Vec3<i64>, BuildingResult);

pub struct TownGen {
    city_gen: CacheGen<StructureGen, Vec2<i64>, CityGenOut>,
    building_gen: CacheGen<StructureGen, Vec2<i64>, BuildingGenOut>,
}

impl TownGen {
    pub fn new() -> Self {
        Self {
            city_gen: CacheGen::new(StructureGen::new(
                350, // freq
                256, // warp
                new_seed(), // seed
                dist_by_euc, // distance function
            ), 4096),
            building_gen: CacheGen::new(StructureGen::new(
                34, // freq
                16, // warp
                new_seed(), // seed
                dist_by_euc, // distance function
            ), 4096),
        }
    }
}

impl StructureGen {
    fn gen_city(&self, pos: Vec2<i64>, overworld_gen: &OverworldGen) -> CityGenOut {
        let overworld = overworld_gen.sample(pos, &());

        (
            pos,
            // Town
            if overworld.dry < 0.2 && overworld.land > 0.0 && self.throw_dice(pos, 0) % 50 < 20 {
                CityResult::Town
            // Pyramid
            } else if overworld.temp > 0.45 && overworld.land > 0.0 && overworld.dry > 0.05 && overworld.land < 0.5 && self.throw_dice(pos, 0) % 50 < 20 {
                CityResult::Pyramid { height: 30 + self.throw_dice(pos, 0) % 40, z: overworld.z_alt as i64 }
            // Wilderness
            } else {
                CityResult::None
            }
        )
    }

    fn gen_building(&self, pos: Vec2<i64>, (city_gen, overworld_gen): &(&StructureGen, &OverworldGen)) -> BuildingGenOut {
        let overworld = overworld_gen.sample(pos, &());

        let city = city_gen.sample(pos, &(*overworld_gen, StructureGen::gen_city));

        // Buildings
        match city {
            // Town
            (city_pos, CityResult::Town) => {
                (
                    Vec3::new(pos.x, pos.y, overworld.z_alt as i64 - 8),
                    if overworld.dry > 0.005 {
                        BuildingResult::House {
                            idx: self.throw_dice(pos, 1) as usize % BUILDINGS.len(),
                            unit_x: Vec2::unit_x() * if self.throw_dice(pos, 2) & 2 == 0 { 1 } else { -1 },
                            unit_y: Vec2::unit_y() * if self.throw_dice(pos, 2) & 2 == 0 { 1 } else { -1 },
                        }
                    } else {
                        BuildingResult::Park
                    },
                )
            },
            // Pyramid
            (city_pos, CityResult::Pyramid { height, z }) => {
                (
                    Vec3::new(city_pos.x, city_pos.y, z),
                    BuildingResult::Pyramid { height },
                )
            },
            // Wilderness
            (city_pos, CityResult::None) => {
                // Rocks
                if self.throw_dice(pos, 0) % 50 < 3 {
                    (
                        Vec3::new(pos.x, pos.y, overworld.z_alt as i64),
                        BuildingResult::Rock,
                    )
                // Trees
                } else if self.throw_dice(pos, 0) % 50 < 30 && overworld.dry > 0.05 && overworld.dry < 0.4 && overworld.land > 0.0 {
                    (
                        Vec3::new(pos.x, pos.y, overworld.z_alt as i64 - 1),
                        BuildingResult::Tree {
                            idx: self.throw_dice(pos, 1) as usize % TREES.len(),
                            unit_x: Vec2::unit_x() * if self.throw_dice(pos, 2) & 2 == 0 { 1 } else { -1 },
                            unit_y: Vec2::unit_y() * if self.throw_dice(pos, 2) & 2 == 0 { 1 } else { -1 },
                        },
                    )
                } else {
                    (
                        Vec3::new(pos.x, pos.y, overworld.z_alt as i64 - 8),
                        BuildingResult::None,
                    )
                }
            }
        }
    }
}

impl Gen<OverworldGen> for TownGen {
    type In = Vec3<i64>;
    type Out = Out;

    fn sample<'a>(&'a self, pos: Vec3<i64>, overworld_gen: &'a OverworldGen) -> Out {
        let pos2d = Vec2::from(pos);

        let mut out = Out {
            is_town: false,
            surface: None,
            block: None,
        };

        let building = self.building_gen.sample(
            pos2d,
            &(&(self.city_gen.internal(), overworld_gen),
            StructureGen::gen_building)
        );

        match building {
            // House
            (building_base, BuildingResult::House { idx, unit_x, unit_y }) => {
                out.is_town = true;
                let building = &BUILDINGS[idx];

                let rel_offs = (pos2d - building_base);

                // Find distance to make path
                if rel_offs.map(|e| e * e).sum() > 16 * 16 {
                    out.surface = Some(match self.building_gen.internal().throw_dice(pos.into(), 0) % 5 {
                        0 => Block::from_byte(109),
                        1 => Block::from_byte(110),
                        2 => Block::from_byte(111),
                        3 => Block::from_byte(112),
                        4 => Block::from_byte(113),
                        _ => Block::AIR,
                    });
                }

                let vox_offs = unit_x * rel_offs.x + unit_y * rel_offs.y + Vec2::from(building.size()).map(|e: u32| e as i64) / 2;
                out.block = building.at(Vec3::new(vox_offs.x, vox_offs.y, pos.z - building_base.z).map(|e| e as u32));
            },
            // Rock
            (rock_base, BuildingResult::Rock) => {
                if (pos - rock_base).map(|e| e * e).sum() < 64 {
                    out.block = Some(Block::STONE);
                }
            },
            // Tree
            (tree_base, BuildingResult::Tree { idx, unit_x, unit_y }) => {
                let tree = &TREES[idx];

                let rel_offs = (pos2d - tree_base);

                let vox_offs = unit_x * rel_offs.x + unit_y * rel_offs.y + Vec2::from(tree.size()).map(|e: u32| e as i64) / 2;
                out.block = tree.at(Vec3::new(vox_offs.x, vox_offs.y, pos.z - tree_base.z).map(|e| e as u32));
            },
            // Pyramid
            (pyramid_base, BuildingResult::Pyramid { height }) => {
                let rel_offs = (pos2d - pyramid_base);

                let pyramid_h = (pyramid_base.z + height as i64) - rel_offs.map(|e| e.abs()).reduce_max();

                if
                    pos.z < pyramid_h &&
                    !(rel_offs.map(|e| e.abs()).reduce_min() < 2 && (pos.z) % 20 < 4) &&
                    !(pos.z < pyramid_h - 6 && (pos.z) % 20 < 16)
                {
                    out.block = Some(Block::SAND);
                }
            },
            // Nothing
            _ => {},
        }

        out
    }
}
