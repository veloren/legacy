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

lazy_static! {
    static ref BUILDINGS: Vec<HeterogeneousData> = load_buildings();
}

// <--- END MESS --->

#[derive(Copy, Clone)]
pub struct Out {
    pub is_town: bool,
    pub surface: Option<Block>,
    pub block: Option<Block>,
}

#[derive(Copy, Clone)]
enum BuildingResult {
    Building {
        base: Vec3<i64>,
        idx: usize,
        unit_x: Vec2<i64>,
        unit_y: Vec2<i64>,
    },
    Park,
}

type CityGenOut = Option<Vec2<i64>>;
type BuildingGenOut = Option<BuildingResult>;

pub struct TownGen {
    city_gen: CacheGen<StructureGen, Vec2<i64>, CityGenOut>,
    building_gen: CacheGen<StructureGen, Vec2<i64>, BuildingGenOut>,
}

impl TownGen {
    pub fn new() -> Self {
        Self {
            city_gen: CacheGen::new(StructureGen::new(
                256, // freq
                256, // warp
                new_seed(), // seed
                dist_by_euc, // distance function
            ), 4096),
            building_gen: CacheGen::new(StructureGen::new(
                32, // freq
                20, // warp
                new_seed(), // seed
                dist_by_euc, // distance function
            ), 4096),
        }
    }
}

impl StructureGen {
    fn gen_city(&self, pos: Vec2<i64>, overworld_gen: &OverworldGen) -> CityGenOut {
        let overworld = overworld_gen.sample(pos, &());

        if overworld.land < 0.3 && overworld.land > 0.0 && self.throw_dice(pos, 0) % 50 < 10 {
            Some(pos)
        } else {
            None
        }
    }

    fn gen_building(&self, pos: Vec2<i64>, (city_gen, overworld_gen): &(&StructureGen, &OverworldGen)) -> BuildingGenOut {
        let overworld = overworld_gen.sample(pos, &());

        if let Some(city_pos) = city_gen.sample(pos, &(*overworld_gen, StructureGen::gen_city)) {
            Some(if overworld.dry > 0.005 {
                BuildingResult::Building {
                    base: Vec3::new(pos.x, pos.y, overworld.z_alt as i64 - 8),
                    idx: self.throw_dice(pos, 1) as usize % BUILDINGS.len(),
                    unit_x: Vec2::unit_x() * if self.throw_dice(pos, 2) & 2 == 0 { 1 } else { -1 },
                    unit_y: Vec2::unit_y() * if self.throw_dice(pos, 2) & 2 == 0 { 1 } else { -1 },
                }
            } else {
                BuildingResult::Park
            })
        } else {
            None
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

        self.building_gen.sample(pos2d, &(&(self.city_gen.internal(), overworld_gen), StructureGen::gen_building)).map(|r| {
            out.is_town = true;
            match r {
                BuildingResult::Building { base, idx, unit_x, unit_y } => {
                    let building = &BUILDINGS[idx];

                    let rel_offs = (pos2d - base);

                    // Find distance to make path
                    if rel_offs.map(|e| e * e).sum() > 16 * 16 {
                        out.surface = Some(Block::SAND);
                    }

                    let vox_offs = unit_x * rel_offs.x + unit_y * rel_offs.y + Vec2::from(building.size()).map(|e: u32| e as i64) / 2;
                    out.block = building.at(Vec3::new(vox_offs.x, vox_offs.y, pos.z - base.z).map(|e| e as u32));
                },
                BuildingResult::Park => {},
            }
        });

        out
    }
}
