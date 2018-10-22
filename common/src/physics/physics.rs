// Standard
use std::{clone::Clone, sync::Arc};

// Library
use parking_lot::RwLock;
use vek::*;

// Project
use physics::collision::{Collider, Primitive, ResolutionTti, PLANCK_LENGTH};
use terrain::chunk::{Chunk};
use terrain::{Voxel, VoxelAbsType, VoxelRelType, VoxelRelVec, VoxelAbsVec, VolumeIdxVec};

use Uid;

// Local
use terrain::{Entity, PersState, ChunkMgr};

pub const LENGTH_OF_BLOCK: f32 = 0.3;
const GROUND_GRAVITY: f32 = -9.81;
const BLOCK_SIZE_PLUS_SMALL: f32 = 1.0 + PLANCK_LENGTH;
const BLOCK_HOP_SPEED: f32 = 15.0;
const BLOCK_HOP_MAX: f32 = 0.34;

// estimates the blocks around a entity that are needed during physics calculation.
fn get_nearby(col: &Primitive, dir: Vec3<f32>) -> (/*low:*/VoxelAbsVec , /*high:*/VoxelAbsVec) {
    // get the entity boundrieds and convert them to blocks, then caluclate the entity velocity and adjust it
    // then move the playr up by BLOCK_SIZE_PLUS_SMALL for block hopping

    let scale = Vec3::new(1.0, 1.0, 1.0); //between entities and world
    let dirabs = Vec3::new(dir.x.abs(), dir.y.abs(), dir.z.abs()) / 2.0;
    let area = col.col_approx_abc() + dirabs + scale;

    let pos = col.col_center() + dir / 2.0;
    let low = pos - area;
    let mut high = pos + area;
    // apply Hop correction to high
    high.z += BLOCK_SIZE_PLUS_SMALL;
    // ceil the low and floor the high for dat performance improve
    let low = low.map(|e| e.ceil() as VoxelAbsType - 1);
    let high = high.map(|e| (e.floor() as VoxelAbsType) + 1 + 1); // +1 is for the for loop

    (low, high)
}

#[allow(non_snake_case)]
pub fn tick<
    'a,
    CP: Send + Sync + 'static,
    EP: Send + Sync + 'static,
    I: Iterator<Item = (&'a Uid, &'a Arc<RwLock<Entity<EP>>>)>,
>(
    entities: I,
    chunk_mgr: &ChunkMgr<CP>,
    chunk_size: VoxelRelVec,
    dt: f32,
) {
    // TODO: use const support once we use Vek
    let ENTITY_MIDDLE_OFFSET: Vec3<f32> = Vec3::new(0.0, 0.0, 0.9);
    let ENTITY_RADIUS: Vec3<f32> = Vec3::new(0.45, 0.45, 0.9);
    let SMALLER_THAN_BLOCK_GOING_DOWN: Vec3<f32> = Vec3::new(0.0, 0.0, -0.1);
    let CONTROL_IN_AIR: Vec3<f32> = Vec3::new(0.17, 0.17, 0.0);
    let ENTITY_ACC: Vec3<f32> = Vec3::new(32.0 / LENGTH_OF_BLOCK, 32.0 / LENGTH_OF_BLOCK, 200.0 / LENGTH_OF_BLOCK);
    let FRICTION_ON_GROUND: Vec3<f32> = Vec3::new(0.0015, 0.0015, 0.0015);
    let FRICTION_IN_AIR: Vec3<f32> = Vec3::new(0.2, 0.2, 0.78);

    for (.., entity) in entities {
        let mut entity = entity.write();

        // Gravity
        let gravity_acc = Vec3::new(0.0, 0.0, GROUND_GRAVITY / LENGTH_OF_BLOCK);
        let middle = *entity.pos() + ENTITY_MIDDLE_OFFSET;
        let radius = ENTITY_RADIUS;

        let mut entity_prim = Primitive::new_cuboid(middle, radius);

        // generate primitives from volsample
        //TODO: Fix calculation! FIXME
        let (low, high) = get_nearby(&entity_prim, Vec3::new(0.0, 0.0, 0.0));
        println!("low: {}, high: {}", &low, &high);
        let volsample = chunk_mgr.try_get_sample(low, high);
        if let Err(_) = volsample {
            continue; //skip this entity, because not all chunks are loaded
        }
        let volsample = volsample.unwrap();
        let mut nearby_primitves = Vec::new();
        for (pos, b) in volsample.iter() {
            if b.is_solid() {
                let entity = Primitive::new_cuboid(pos.map(|e| e as f32) + Vec3::new(0.5, 0.5, 0.5), Vec3::new(0.5, 0.5, 0.5));
                nearby_primitves.push(entity);
            }
        }

        // is standing on ground to jump
        let mut on_ground = false;
        let can_jump_prim = Primitive::new_cuboid(middle, radius);
        for prim in &nearby_primitves {
            let res = prim.time_to_impact(&can_jump_prim, &SMALLER_THAN_BLOCK_GOING_DOWN);
            if let Some(ResolutionTti::WillCollide { tti, .. }) = res {
                if tti < PLANCK_LENGTH * 2.0 {
                    // something really small
                    on_ground = true;
                    break;
                }
            }
        }

        let mut wanted_ctrl_acc = *entity.ctrl_acc();
        // TODO: move to client
        // apply checking if player can conrol (touches ground) out this in client instead of physics
        if !on_ground {
            wanted_ctrl_acc *= CONTROL_IN_AIR;
        }

        // TODO: move to client
        let wanted_ctrl_acc_length = Vec3::new(wanted_ctrl_acc.x, wanted_ctrl_acc.y, 0.0).magnitude();
        if wanted_ctrl_acc_length > 1.0 {
            wanted_ctrl_acc.x /= wanted_ctrl_acc_length;
            wanted_ctrl_acc.y /= wanted_ctrl_acc_length;
        }

        // multiply by entity speed
        wanted_ctrl_acc *= ENTITY_ACC;

        // calc acc
        let acc = wanted_ctrl_acc + gravity_acc;

        // apply acc to vel
        *entity.vel_mut() += acc * dt;

        // apply friction to vel
        let fric_fac = if on_ground {
            FRICTION_ON_GROUND.map(|e| e.powf(dt))
        } else {
            FRICTION_IN_AIR.map(|e| e.powf(dt))
        };
        *entity.vel_mut() *= fric_fac;

        let mut velocity = *entity.vel() * dt;
        //debug!("velocity: {}", velocity);

        // movement can be executed in max 3 steps because we are using TTI
        for _ in 0..3 {
            if velocity.magnitude() < PLANCK_LENGTH {
                break;
            }

            // collision with terrain
            let mut tti = 1.0; // 1.0 = full tick
            let mut normal = Vec3::new(0.0, 0.0, 0.0);

            for prim in &nearby_primitves {
                let r = prim.time_to_impact(&entity_prim, &velocity);
                if let Some(r) = r {
                    //info!("colliding in tti: {:?}", r);
                    if let ResolutionTti::WillCollide {
                        tti: ltti,
                        normal: lnormal,
                    } = r
                    {
                        if ltti <= tti {
                            //debug!("colliding in tti: {}, normal {}", ltti, lnormal);
                            if lnormal.magnitude() < normal.magnitude() || normal.magnitude() < 0.1 || ltti < tti {
                                // when tti is same but we have less normal we switch
                                //info!("set normal to: {}", lnormal);
                                // if there is a collission with 2 and one with 1 block we first solve the single one
                                normal = lnormal;
                            }
                            tti = ltti;
                        }
                    }
                }
            }

            if tti > 0.0 {
                let movement = velocity * tti;
                if tti < 1.0 {
                    info!("total valid tti: {}", tti);
                    debug!("move by: {}", movement);
                }
                entity_prim.move_by(&movement);
                velocity -= movement;
            }

            if normal.x != 0.0 || normal.y != 0.0 {
                // block hopping
                let mut auto_jump_prim = entity_prim.clone();
                auto_jump_prim.move_by(&Vec3::new(0.0, 0.0, BLOCK_SIZE_PLUS_SMALL));
                let mut collision_after_hop = false;
                for prim in &nearby_primitves {
                    let res = prim.resolve_col(&auto_jump_prim);
                    if let Some(..) = res {
                        collision_after_hop = true;
                        break;
                    }
                }
                if collision_after_hop {
                    if normal.x != 0.0 {
                        debug!("full stop x");
                        velocity.x = 0.0;
                        entity.vel_mut().x = 0.0;
                    }
                    if normal.y != 0.0 {
                        debug!("full stop y");
                        velocity.y = 0.0;
                        entity.vel_mut().y = 0.0;
                    }
                } else {
                    let mut smoothmove = BLOCK_HOP_SPEED * dt;
                    if smoothmove > BLOCK_HOP_MAX {
                        smoothmove = BLOCK_HOP_MAX;
                    };
                    entity_prim.move_by(&Vec3::new(0.0, 0.0, smoothmove));
                }
            }
            if normal.z != 0.0 {
                //debug!("full stop z");
                velocity.z = 0.0;
                entity.vel_mut().z = 0.0;
            }
        }

        // am i stuck check
        let mut entity_prim_stuck = entity_prim.clone();
        entity_prim_stuck.scale_by(0.9);
        for prim in nearby_primitves {
            let res = prim.resolve_col(&entity_prim_stuck);
            if let Some(..) = res {
                warn!("entity is stuck!");
                entity_prim.move_by(&Vec3::new(0.0, 0.0, BLOCK_SIZE_PLUS_SMALL));
                break;
            }
        }

        let cd = entity_prim.col_center().map(|e| e as VoxelAbsType);
        if !chunk_mgr.exists_block(cd) {
            *entity.vel_mut() = Vec3::broadcast(0.0);
            continue; //skip applying
        }

        // apply
        *entity.pos_mut() = entity_prim.col_center() - ENTITY_MIDDLE_OFFSET;
    }
}
