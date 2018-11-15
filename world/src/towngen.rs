// Standard
use std::{
    ops::{Add, Div, Mul, Sub},
    process,
};

// Library
use dot_vox;
use vek::*;

// Project
use common::terrain::chunk::Block;

// Local
use cachegen::CacheGen;
use new_seed;
use overworldgen::{Out as OverworldOut, OverworldGen};
use util::structure::{dist_by_euc, StructureGen};
use Gen;

// <--- BEGIN MESS --->

// Project
use common::{
    get_asset_dir, get_asset_path,
    terrain::{chunk::HeterogeneousData, ConstructVolume, ReadVolume, ReadWriteVolume, Volume, Voxel},
};

// TODO: Replace this with a superior voxel loading system
// Maybe include_bytes! these files into the executable?
// Might limit modding

fn dot_vox_to_hetero(vox: dot_vox::DotVoxData) -> HeterogeneousData {
    match vox.models.first() {
        Some(model) => {
            let size = Vec3::new(model.size.x, model.size.y, model.size.z).map(|e| e as u32);
            let mut chunk = HeterogeneousData::filled(size, Block::AIR);

            for ref v in model.voxels.iter() {
                let pos = Vec3::new(v.x as u32, v.y as u32, v.z as u32);
                chunk.set_at(
                    pos,
                    match v.i {
                        7...9 | 224...255 => Block::AIR,
                        i => Block::from_byte(i),
                    },
                );
            }

            chunk
        },
        None => HeterogeneousData::filled(Vec3::zero(), Block::AIR),
    }
}

fn asset_load_error(err_msg: &'static str) -> dot_vox::DotVoxData {
    println!("{}", err_msg);
    println!("An asset could not be found.");
    println!(
        "Please ensure that the asset directory is located at '{}' and is populated.",
        get_asset_dir().to_str().unwrap()
    );
    println!("Veloren will now exit.");
    process::exit(1);
}

fn load_buildings() -> Vec<HeterogeneousData> {
    let mut buildings = vec![];

    buildings.push(dot_vox_to_hetero(
        dot_vox::load(
            get_asset_path("world/Structures/Human/Houses/16x16x16/Turqoise/turq1.vox")
                .to_str()
                .unwrap(),
        )
        .unwrap_or_else(asset_load_error),
    ));
    buildings.push(dot_vox_to_hetero(
        dot_vox::load(
            get_asset_path("world/Structures/Human/Houses/16x16x16/Turqoise/turq2.vox")
                .to_str()
                .unwrap(),
        )
        .unwrap_or_else(asset_load_error),
    ));
    buildings.push(dot_vox_to_hetero(
        dot_vox::load(
            get_asset_path("world/Structures/Human/Houses/16x16x16/Turqoise/turq3.vox")
                .to_str()
                .unwrap(),
        )
        .unwrap_or_else(asset_load_error),
    ));
    //buildings.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Structures/Human/Houses/16x16x16/Turqoise/turq4.vox").unwrap()));
    //buildings.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Structures/Human/Houses/16x16x16/Turqoise/turq5.vox").unwrap()));
    buildings.push(dot_vox_to_hetero(
        dot_vox::load(
            get_asset_path("world/Structures/Human/Houses/16x16x16/Blue/blue1.vox")
                .to_str()
                .unwrap(),
        )
        .unwrap_or_else(asset_load_error),
    ));
    buildings.push(dot_vox_to_hetero(
        dot_vox::load(
            get_asset_path("world/Structures/Human/Houses/16x16x16/Blue/blue2.vox")
                .to_str()
                .unwrap(),
        )
        .unwrap_or_else(asset_load_error),
    ));
    buildings.push(dot_vox_to_hetero(
        dot_vox::load(
            get_asset_path("world/Structures/Human/Houses/16x16x16/Blue/blue3.vox")
                .to_str()
                .unwrap(),
        )
        .unwrap_or_else(asset_load_error),
    ));
    //buildings.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Structures/Human/Houses/16x16x16/Blue/blue4.vox").unwrap()));
    //buildings.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Structures/Human/Houses/16x16x16/Blue/blue5.vox").unwrap()));
    buildings.push(dot_vox_to_hetero(
        dot_vox::load(
            get_asset_path("world/Structures/Human/Houses/16x16x16/Red/1R.vox")
                .to_str()
                .unwrap(),
        )
        .unwrap_or_else(asset_load_error),
    ));
    buildings.push(dot_vox_to_hetero(
        dot_vox::load(
            get_asset_path("world/Structures/Human/Houses/16x16x16/Red/2R.vox")
                .to_str()
                .unwrap(),
        )
        .unwrap_or_else(asset_load_error),
    ));
    buildings.push(dot_vox_to_hetero(
        dot_vox::load(
            get_asset_path("world/Structures/Human/Houses/16x16x16/Red/3R.vox")
                .to_str()
                .unwrap(),
        )
        .unwrap_or_else(asset_load_error),
    ));
    //buildings.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Structures/Human/Houses/16x16x16/Red/4R.vox").unwrap()));
    //buildings.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Structures/Human/Houses/16x16x16/Red/5R.vox").unwrap()));
    buildings.push(dot_vox_to_hetero(
        dot_vox::load(
            get_asset_path("world/Structures/Human/Houses/16x16x16/Green/green1.vox")
                .to_str()
                .unwrap(),
        )
        .unwrap_or_else(asset_load_error),
    ));
    buildings.push(dot_vox_to_hetero(
        dot_vox::load(
            get_asset_path("world/Structures/Human/Houses/16x16x16/Green/green2.vox")
                .to_str()
                .unwrap(),
        )
        .unwrap_or_else(asset_load_error),
    ));
    //buildings.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Structures/Human/Houses/16x16x16/Green/green3.vox").unwrap()));
    //buildings.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Structures/Human/Houses/16x16x16/Green/green4.vox").unwrap()));
    //buildings.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Structures/Human/Houses/16x16x16/Green/green5.vox").unwrap()));

    //buildings.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Structures/Human/townhall.vox").unwrap()));
    //buildings.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Structures/Human/tower.vox").unwrap()));

    buildings.push(dot_vox_to_hetero(
        dot_vox::load(
            get_asset_path("world/Trees/Veloren_Trees/Birches/Birch_1.vox")
                .to_str()
                .unwrap(),
        )
        .unwrap_or_else(asset_load_error),
    ));
    buildings.push(dot_vox_to_hetero(
        dot_vox::load(
            get_asset_path("world/Trees/Veloren_Trees/Poplars/1.vox")
                .to_str()
                .unwrap(),
        )
        .unwrap_or_else(asset_load_error),
    ));
    //buildings.push(dot_vox_to_hetero(dot_vox::load("../assets/world/Trees/Veloren_Trees/Willows/1.vox").unwrap()));

    buildings
}

fn load_trees_temperate() -> Vec<HeterogeneousData> {
    let mut trees = vec![];

    trees.push(dot_vox_to_hetero(
        dot_vox::load(
            get_asset_path("world/Trees/Veloren_Trees_Purple/Oaks/Oak1.vox")
                .to_str()
                .unwrap(),
        )
        .unwrap_or_else(asset_load_error),
    ));
    trees.push(dot_vox_to_hetero(
        dot_vox::load(
            get_asset_path("world/Trees/Veloren_Trees_Purple/Oaks/Oak2.vox")
                .to_str()
                .unwrap(),
        )
        .unwrap_or_else(asset_load_error),
    ));
    trees.push(dot_vox_to_hetero(
        dot_vox::load(
            get_asset_path("world/Trees/Veloren_Trees_Purple/Oaks/Oak3.vox")
                .to_str()
                .unwrap(),
        )
        .unwrap_or_else(asset_load_error),
    ));
    trees.push(dot_vox_to_hetero(
        dot_vox::load(
            get_asset_path("world/Trees/Veloren_Trees_Purple/Oaks/Oak4.vox")
                .to_str()
                .unwrap(),
        )
        .unwrap_or_else(asset_load_error),
    ));
    trees.push(dot_vox_to_hetero(
        dot_vox::load(
            get_asset_path("world/Trees/Veloren_Trees_Purple/Oaks/Oak5.vox")
                .to_str()
                .unwrap(),
        )
        .unwrap_or_else(asset_load_error),
    ));
    trees.push(dot_vox_to_hetero(
        dot_vox::load(
            get_asset_path("world/Trees/Veloren_Trees_Purple/Oaks/Oak6.vox")
                .to_str()
                .unwrap(),
        )
        .unwrap_or_else(asset_load_error),
    ));
    trees.push(dot_vox_to_hetero(
        dot_vox::load(
            get_asset_path("world/Trees/Veloren_Trees_Purple/Oaks/Oak7.vox")
                .to_str()
                .unwrap(),
        )
        .unwrap_or_else(asset_load_error),
    ));
    trees.push(dot_vox_to_hetero(
        dot_vox::load(
            get_asset_path("world/Trees/Veloren_Trees_Purple/Oaks/Oak8.vox")
                .to_str()
                .unwrap(),
        )
        .unwrap_or_else(asset_load_error),
    ));
    trees.push(dot_vox_to_hetero(
        dot_vox::load(
            get_asset_path("world/Trees/Veloren_Trees_Purple/Oaks/Oak9.vox")
                .to_str()
                .unwrap(),
        )
        .unwrap_or_else(asset_load_error),
    ));
    trees.push(dot_vox_to_hetero(
        dot_vox::load(
            get_asset_path("world/Trees/Veloren_Trees_Purple/Oaks/Oak10.vox")
                .to_str()
                .unwrap(),
        )
        .unwrap_or_else(asset_load_error),
    ));

    trees.push(dot_vox_to_hetero(
        dot_vox::load(
            get_asset_path("world/Trees/Veloren_Trees_Purple/Poplars/1.vox")
                .to_str()
                .unwrap(),
        )
        .unwrap_or_else(asset_load_error),
    ));
    trees.push(dot_vox_to_hetero(
        dot_vox::load(
            get_asset_path("world/Trees/Veloren_Trees_Purple/Poplars/2.vox")
                .to_str()
                .unwrap(),
        )
        .unwrap_or_else(asset_load_error),
    ));
    trees.push(dot_vox_to_hetero(
        dot_vox::load(
            get_asset_path("world/Trees/Veloren_Trees_Purple/Poplars/3.vox")
                .to_str()
                .unwrap(),
        )
        .unwrap_or_else(asset_load_error),
    ));
    trees.push(dot_vox_to_hetero(
        dot_vox::load(
            get_asset_path("world/Trees/Veloren_Trees_Purple/Poplars/4.vox")
                .to_str()
                .unwrap(),
        )
        .unwrap_or_else(asset_load_error),
    ));
    trees.push(dot_vox_to_hetero(
        dot_vox::load(
            get_asset_path("world/Trees/Veloren_Trees_Purple/Poplars/5.vox")
                .to_str()
                .unwrap(),
        )
        .unwrap_or_else(asset_load_error),
    ));
    trees.push(dot_vox_to_hetero(
        dot_vox::load(
            get_asset_path("world/Trees/Veloren_Trees_Purple/Poplars/6.vox")
                .to_str()
                .unwrap(),
        )
        .unwrap_or_else(asset_load_error),
    ));
    trees.push(dot_vox_to_hetero(
        dot_vox::load(
            get_asset_path("world/Trees/Veloren_Trees_Purple/Poplars/7.vox")
                .to_str()
                .unwrap(),
        )
        .unwrap_or_else(asset_load_error),
    ));
    trees.push(dot_vox_to_hetero(
        dot_vox::load(
            get_asset_path("world/Trees/Veloren_Trees_Purple/Poplars/8.vox")
                .to_str()
                .unwrap(),
        )
        .unwrap_or_else(asset_load_error),
    ));
    trees.push(dot_vox_to_hetero(
        dot_vox::load(
            get_asset_path("world/Trees/Veloren_Trees_Purple/Poplars/9.vox")
                .to_str()
                .unwrap(),
        )
        .unwrap_or_else(asset_load_error),
    ));
    trees.push(dot_vox_to_hetero(
        dot_vox::load(
            get_asset_path("world/Trees/Veloren_Trees_Purple/Poplars/10.vox")
                .to_str()
                .unwrap(),
        )
        .unwrap_or_else(asset_load_error),
    ));

    trees.push(dot_vox_to_hetero(
        dot_vox::load(
            get_asset_path("world/Trees/Veloren_Trees_Purple/Willows/1.vox")
                .to_str()
                .unwrap(),
        )
        .unwrap_or_else(asset_load_error),
    ));
    trees.push(dot_vox_to_hetero(
        dot_vox::load(
            get_asset_path("world/Trees/Veloren_Trees_Purple/Willows/2.vox")
                .to_str()
                .unwrap(),
        )
        .unwrap_or_else(asset_load_error),
    ));

    trees
}

fn load_trees_tropical() -> Vec<HeterogeneousData> {
    let mut trees = vec![];

    trees.push(dot_vox_to_hetero(
        dot_vox::load(
            get_asset_path("world/Trees/Veloren_Trees_Purple/Birches/Birch_1.vox")
                .to_str()
                .unwrap(),
        )
        .unwrap_or_else(asset_load_error),
    ));
    trees.push(dot_vox_to_hetero(
        dot_vox::load(
            get_asset_path("world/Trees/Veloren_Trees_Purple/Birches/Birch_2.vox")
                .to_str()
                .unwrap(),
        )
        .unwrap_or_else(asset_load_error),
    ));
    trees.push(dot_vox_to_hetero(
        dot_vox::load(
            get_asset_path("world/Trees/Veloren_Trees_Purple/Birches/Birch_3.vox")
                .to_str()
                .unwrap(),
        )
        .unwrap_or_else(asset_load_error),
    ));
    trees.push(dot_vox_to_hetero(
        dot_vox::load(
            get_asset_path("world/Trees/Veloren_Trees_Purple/Birches/Birch_4.vox")
                .to_str()
                .unwrap(),
        )
        .unwrap_or_else(asset_load_error),
    ));
    trees.push(dot_vox_to_hetero(
        dot_vox::load(
            get_asset_path("world/Trees/Veloren_Trees_Purple/Birches/Birch_5.vox")
                .to_str()
                .unwrap(),
        )
        .unwrap_or_else(asset_load_error),
    ));
    trees.push(dot_vox_to_hetero(
        dot_vox::load(
            get_asset_path("world/Trees/Veloren_Trees_Purple/Birches/Birch_6.vox")
                .to_str()
                .unwrap(),
        )
        .unwrap_or_else(asset_load_error),
    ));
    trees.push(dot_vox_to_hetero(
        dot_vox::load(
            get_asset_path("world/Trees/Veloren_Trees_Purple/Birches/Birch_7.vox")
                .to_str()
                .unwrap(),
        )
        .unwrap_or_else(asset_load_error),
    ));
    trees.push(dot_vox_to_hetero(
        dot_vox::load(
            get_asset_path("world/Trees/Veloren_Trees_Purple/Birches/Birch_8.vox")
                .to_str()
                .unwrap(),
        )
        .unwrap_or_else(asset_load_error),
    ));
    trees.push(dot_vox_to_hetero(
        dot_vox::load(
            get_asset_path("world/Trees/Veloren_Trees_Purple/Birches/Birch_9.vox")
                .to_str()
                .unwrap(),
        )
        .unwrap_or_else(asset_load_error),
    ));
    trees.push(dot_vox_to_hetero(
        dot_vox::load(
            get_asset_path("world/Trees/Veloren_Trees_Purple/Birches/Birch_10.vox")
                .to_str()
                .unwrap(),
        )
        .unwrap_or_else(asset_load_error),
    ));

    trees
}

const IDX_BUILDINGS: usize = 0;
const IDX_TREES_TEMPERATE: usize = 1;
const IDX_TREES_TROPICAL: usize = 2;
lazy_static! {
    static ref VOXEL_MODELS: [Vec<HeterogeneousData>; 3] =
        [load_buildings(), load_trees_temperate(), load_trees_tropical(),];
}

// <--- END MESS --->

#[derive(Copy, Clone)]
pub struct Out {
    pub surface: Option<Block>,
    pub block: Option<Block>,
}

#[derive(Copy, Clone)]
#[allow(dead_code)]
enum ForestKind {
    Tropical,
    Temperate,
    Taiga,
    Desert,
}

#[derive(Copy, Clone)]
enum CityResult {
    Town,
    Pyramid { height: u64, z: i64 },
    Forest { kind: ForestKind, density: u64 },
    None,
}

#[derive(Copy, Clone)]
pub enum BuildingResult {
    House {
        model: &'static HeterogeneousData,
        unit_x: Vec2<i64>,
        unit_y: Vec2<i64>,
    },
    Park,
    Tree {
        model: &'static HeterogeneousData,
        leaf_block: Block,
        scale_inv: u64,
        unit_x: Vec2<i64>,
        unit_y: Vec2<i64>,
    },
    Rock,
    Pyramid {
        height: u64,
    },
    None,
}

type CityGenOut = (Vec2<i64>, CityResult);
type BuildingGenOut = (Vec3<i64>, BuildingResult);

pub struct TownGen {
    city_gen: CacheGen<StructureGen<CityGenOut>, Vec2<i64>, (CityGenOut, [CityGenOut; 9])>,
    building_gen: CacheGen<StructureGen<BuildingGenOut>, Vec2<i64>, (BuildingGenOut, [BuildingGenOut; 9])>,
}

pub type InvariantZ = (BuildingGenOut, [BuildingGenOut; 9]);

impl TownGen {
    pub fn new() -> Self {
        Self {
            city_gen: CacheGen::new(
                StructureGen::new(
                    350,         // freq
                    256,         // warp
                    new_seed(),  // seed
                    dist_by_euc, // distance function
                ),
                4096,
            ),
            building_gen: CacheGen::new(
                StructureGen::new(
                    24,          // freq
                    12,          // warp
                    new_seed(),  // seed
                    dist_by_euc, // distance function
                ),
                4096,
            ),
        }
    }

    pub fn get_invariant_z<'a>(
        &'a self,
        pos: Vec2<i64>,
        (_overworld, overworld_gen): (&'a OverworldOut, &'a OverworldGen),
    ) -> InvariantZ {
        self.building_gen.sample(
            pos,
            &(&(self.city_gen.internal(), overworld_gen), StructureGen::gen_building),
        )
    }
}

impl StructureGen<CityGenOut> {
    fn gen_city(&self, pos: Vec2<i64>, overworld_gen: &OverworldGen) -> CityGenOut {
        let overworld = overworld_gen.sample(pos, &());

        (
            pos,
            // Town
            if overworld.dry < 0.2
                && overworld.z_alt > overworld.z_sea
                && overworld.land < 0.5
                && self.throw_dice(pos, 0) & 0xFF < 26
            {
                CityResult::Town
            // Pyramid
            } else if overworld.temp > 0.6
                && overworld.z_alt > overworld.z_sea
                && overworld.dry > 0.05
                && overworld.land < 0.5
                && self.throw_dice(pos, 1) & 0xFF < 26
            {
                CityResult::Pyramid {
                    height: 64 + self.throw_dice(pos, 0) % 64,
                    z: overworld.z_alt as i64,
                }
            // Forest
            } else if overworld.dry < 0.3 && self.throw_dice(pos, 0) & 0xFF < 128 {
                CityResult::Forest {
                    kind: if overworld.temp > 0.5 {
                        ForestKind::Tropical
                    } else if overworld.temp < -0.5 {
                        ForestKind::Taiga
                    } else {
                        ForestKind::Temperate
                    },
                    density: 50 + self.throw_dice(pos, 0) % 206,
                }
            // Empty
            } else {
                CityResult::None
            },
        )
    }
}

impl StructureGen<BuildingGenOut> {
    fn gen_building(
        &self,
        pos: Vec2<i64>,
        (city_gen, overworld_gen): &(&StructureGen<CityGenOut>, &OverworldGen),
    ) -> BuildingGenOut {
        let overworld = overworld_gen.sample(pos, &());

        let city = city_gen.sample(pos, &(*overworld_gen, StructureGen::gen_city)).0;

        // Buildings
        match city {
            // Town
            (_city_pos, CityResult::Town) => (
                Vec3::new(pos.x, pos.y, overworld.z_alt as i64 - 8),
                if overworld.dry > 0.005 && overworld.z_alt > overworld.z_sea && self.throw_dice(pos, 0) % 256 < 128 {
                    BuildingResult::House {
                        model: &VOXEL_MODELS[IDX_BUILDINGS]
                            [self.throw_dice(pos, 1) as usize % VOXEL_MODELS[IDX_BUILDINGS].len()],
                        unit_x: Vec2::unit_x() * if self.throw_dice(pos, 2) & 2 == 0 { 1 } else { -1 },
                        unit_y: Vec2::unit_y() * if self.throw_dice(pos, 3) & 2 == 0 { 1 } else { -1 },
                    }
                } else {
                    BuildingResult::Park
                },
            ),
            // Pyramid
            (city_pos, CityResult::Pyramid { height, z }) => {
                (Vec3::new(city_pos.x, city_pos.y, z), BuildingResult::Pyramid { height })
            },
            // Forest
            (_city_pos, CityResult::Forest { kind, density }) => (
                Vec3::new(pos.x, pos.y, overworld.z_alt as i64 - 1),
                if overworld.dry > 0.005 && overworld.z_alt > overworld.z_sea && self.throw_dice(pos, 1) % 256 < density
                {
                    let model_group_idx = match kind {
                        ForestKind::Temperate => IDX_TREES_TEMPERATE,
                        ForestKind::Tropical => IDX_TREES_TROPICAL,
                        _ => IDX_TREES_TEMPERATE,
                    };

                    BuildingResult::Tree {
                        model: &VOXEL_MODELS[model_group_idx]
                            [self.throw_dice(pos, 2) as usize % VOXEL_MODELS[model_group_idx].len()],
                        leaf_block: Block::gradient2(
                            Block::GRAD2_A_LEAF0,
                            Block::GRAD2_B_LEAF1,
                            (overworld.temp.sub(0.65).mul(4.0))
                                .max(0.0)
                                .min(1.0)
                                .add(overworld.temp_vari * 0.7)
                                .max(0.0)
                                .min(1.0)
                                .mul(32.0) as u8,
                        ),
                        scale_inv: 256 + self.throw_dice(pos, 3) % 256,
                        unit_x: Vec2::unit_x() * if self.throw_dice(pos, 4) & 2 == 0 { 1 } else { -1 },
                        unit_y: Vec2::unit_y() * if self.throw_dice(pos, 5) & 2 == 0 { 1 } else { -1 },
                    }
                } else {
                    BuildingResult::None
                },
            ),
            // Empty
            (_city_pos, CityResult::None) => {
                // Rocks
                if self.throw_dice(pos, 0) % 50 < 3 {
                    (Vec3::new(pos.x, pos.y, overworld.z_alt as i64), BuildingResult::Rock)
                // Trees
                } else if self.throw_dice(pos, 0) & 0xFF < 150
                    && overworld.temp < 0.35
                    && overworld.dry > 0.05
                    && overworld.dry < 0.4
                    && overworld.z_alt > overworld.z_sea
                {
                    (
                        Vec3::new(pos.x, pos.y, overworld.z_alt as i64 - 1),
                        BuildingResult::Tree {
                            model: &VOXEL_MODELS[IDX_TREES_TEMPERATE]
                                [self.throw_dice(pos, 1) as usize % VOXEL_MODELS[IDX_TREES_TEMPERATE].len()],
                            leaf_block: Block::gradient2(
                                Block::GRAD2_A_LEAF0,
                                Block::GRAD2_B_LEAF1,
                                (overworld.temp.sub(0.65).mul(4.0))
                                    .max(0.0)
                                    .min(1.0)
                                    .add(overworld.temp_vari * 0.7)
                                    .max(0.0)
                                    .min(1.0)
                                    .mul(32.0) as u8,
                            ),
                            scale_inv: 256 + self.throw_dice(pos, 3) % 256,
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
            },
        }
    }
}

impl<'a> Gen<(&'a InvariantZ, &'a OverworldOut, &'a OverworldGen)> for TownGen {
    type In = Vec3<i64>;
    type Out = Out;

    fn sample<'b>(
        &'b self,
        pos: Vec3<i64>,
        (building, overworld, _overworld_gen): &'b (&'a InvariantZ, &'a OverworldOut, &'a OverworldGen),
    ) -> Out {
        let pos2d = Vec2::from(pos);

        let mut out = Out {
            surface: None,
            block: None,
        };

        // Non-specific effects
        for building in building.1.iter() {
            // Exit early if we already found a suitable block for this position from the surrounding structures
            if out.block.map(|block| block != Block::AIR).unwrap_or(false) {
                break;
            }

            match building {
                // House
                &(building_base, BuildingResult::House { model, unit_x, unit_y }) => {
                    let rel_offs: Vec2<i64> = pos2d - building_base;

                    let vox_offs =
                        unit_x * rel_offs.x + unit_y * rel_offs.y + Vec2::from(model.size()).map(|e: u32| e as i64) / 2;
                    out.block = model.at(Vec3::new(vox_offs.x, vox_offs.y, pos.z - building_base.z).map(|e| e as u32));
                },
                // Rock
                &(rock_base, BuildingResult::Rock) => {
                    if (pos - rock_base).map(|e| e * e).sum() < 64 {
                        out.block = Some(Block::STONE);
                    }
                },
                // Tree
                &(
                    tree_base,
                    BuildingResult::Tree {
                        model,
                        leaf_block,
                        scale_inv,
                        unit_x,
                        unit_y,
                    },
                ) => {
                    let rel_offs = pos2d - tree_base;

                    let vox_offs = (unit_x * rel_offs.x + unit_y * rel_offs.y)
                        .mul(scale_inv as i64)
                        .div(256)
                        + Vec2::from(model.size()).map(|e: u32| e as i64) / 2;
                    let block = model.at(Vec3::new(
                        vox_offs.x,
                        vox_offs.y,
                        (pos.z - tree_base.z).mul(scale_inv as i64).div(256),
                    )
                    .map(|e| e as u32));

                    out.block = match block.map(|b| b.material().index()) {
                        Some(15) => Some(leaf_block),
                        Some(b) => Some(Block::from_byte(b)),
                        None => None,
                    };
                },
                // Pyramid
                &(pyramid_base, BuildingResult::Pyramid { height }) => {
                    let rel_offs = pos2d - pyramid_base;

                    let pyramid_h = pyramid_base.z + height as i64 - rel_offs.map(|e| e.abs()).reduce_max();

                    let tpos = pos + Vec3::new(2, 2, 2);
                    let tunnel = (self.building_gen.internal().throw_dice(tpos / Vec3::new(96, 24, 24), 0) % 2 == 0
                        && tpos.x % 96 < 95
                        && tpos.y % 24 < 7
                        && tpos.z % 24 < 7)
                        || (self.building_gen.internal().throw_dice(tpos / Vec3::new(24, 96, 24), 1) % 2 == 0
                            && tpos.x % 24 < 7
                            && tpos.y % 96 < 95
                            && tpos.z % 24 < 7)
                        || (self.building_gen.internal().throw_dice(tpos / Vec3::new(24, 24, 48), 2) % 2 == 0
                            && tpos.x % 24 < 7
                            && tpos.y % 24 < 7
                            && tpos.z % 48 < 47);

                    if pos.z < pyramid_h
                        && !(rel_offs.map(|e| e.abs()).reduce_min() < 2 && (pos.z) % 25 < 4)
                        && !(pos.z < pyramid_h - 6 && tunnel)
                    {
                        out.block = Some(Block::SAND);
                    }
                },
                // Nothing
                _ => {},
            }
        }

        // Specific effects
        match building.0 {
            // House
            (
                building_base,
                BuildingResult::House {
                    model: _,
                    unit_x: _,
                    unit_y: _,
                },
            ) => {
                let rel_offs: Vec2<i64> = pos2d - building_base;

                // Find distance to make path
                if rel_offs.map(|e| e * e).sum() > 10 * 10 {
                    out.surface = Some(match self.building_gen.internal().throw_dice(Vec2::from(pos), 0) % 5 {
                        0 => Block::from_byte(109),
                        1 => Block::from_byte(110),
                        2 => Block::from_byte(111),
                        3 => Block::from_byte(112),
                        4 => Block::from_byte(113),
                        _ => Block::AIR,
                    });
                }
            },
            _ => {},
        }

        out
    }
}
