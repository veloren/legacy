// Standard
use block::Block;
use chunk_conv::{ChunkContainer, ChunkConverter};
use std::{
    clone::Clone,
    collections::HashMap,
    sync::{Arc, RwLock},
};

// Library
use vek::*;

// Project
use collision::{Collider, Primitive, ResolutionTti, PLANCK_LENGTH};
use common::Uid;

// Local
use super::{Chunk, Entity, VolMgr, VolState};

pub const LENGTH_OF_BLOCK: f32 = 0.3;

pub fn tick<
    'a,
    CP: Send + Sync + 'static,
    EP: Send + Sync + 'static,
    I: Iterator<Item = (&'a Uid, &'a Arc<RwLock<Entity<EP>>>)>,
>(
    entities: I,
    chunk_mgr: &VolMgr<Chunk, ChunkContainer<CP>, ChunkConverter, CP>,
    chunk_size: i64,
    dt: f32,
) {
    //consts
    const GROUND_GRAVITY: f32 = -9.81;
    // TODO: coord const support
    let ENTITY_MIDDLE_OFFSET: Vec3<f32> = Vec3::new(0.0, 0.0, 0.9);
    let ENTITY_RADIUS: Vec3<f32> = Vec3::new(0.45, 0.45, 0.9);
    let SMALLER_THAN_BLOCK_GOING_DOWN: Vec3<f32> = Vec3::new(0.0, 0.0, -0.1);
    let CONTROL_IN_AIR: Vec3<f32> = Vec3::new(0.17, 0.17, 0.0);
    let ENTITY_ACC: Vec3<f32> = Vec3::new(32.0 / LENGTH_OF_BLOCK, 32.0 / LENGTH_OF_BLOCK, 200.0 / LENGTH_OF_BLOCK);
    let FRICTION_ON_GROUND: Vec3<f32> = Vec3::new(0.0015, 0.0015, 0.0015);
    let FRICTION_IN_AIR: Vec3<f32> = Vec3::new(0.2, 0.2, 0.78);
    const BLOCK_SIZE_PLUS_SMALL: f32 = 1.0 + PLANCK_LENGTH;
    const BLOCK_HOP_SPEED: f32 = 15.0;
    const BLOCK_HOP_MAX: f32 = 0.34;

    for (.., entity) in entities {
        let mut entity = entity.write().unwrap();

        // Gravity
        let gravity_acc = Vec3::new(0.0, 0.0, GROUND_GRAVITY / LENGTH_OF_BLOCK);
        let middle = *entity.pos() + ENTITY_MIDDLE_OFFSET;
        let radius = ENTITY_RADIUS;

        let mut entity_prim = Primitive::new_cuboid(middle, radius);

        // is standing on ground to jump
        let mut on_ground = false;
        let can_jump_prim = Primitive::new_cuboid(middle, radius);
        let ground_prims = chunk_mgr.get_nearby(&can_jump_prim);
        for prim in ground_prims {
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
        debug!("velocity: {}", velocity);

        // movement can be executed in max 3 steps because we are using TTI
        for _ in 0..3 {
            if velocity.magnitude() < PLANCK_LENGTH {
                break;
            }

            // collision with terrain
            let potential_collision_prims = chunk_mgr.get_nearby_dir(&entity_prim, velocity);
            let mut tti = 1.0; // 1.0 = full tick
            let mut normal = Vec3::new(0.0, 0.0, 0.0);

            for prim in potential_collision_prims {
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
                let potential_collision_prims = chunk_mgr.get_nearby(&auto_jump_prim);
                let mut collision_after_hop = false;
                for prim in potential_collision_prims {
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
                debug!("full stop z");
                velocity.z = 0.0;
                entity.vel_mut().z = 0.0;
            }
        }

        // am i stuck check
        let mut entity_prim_stuck = entity_prim.clone();
        entity_prim_stuck.scale_by(0.9);
        let stuck_check = chunk_mgr.get_nearby(&entity_prim_stuck);
        for prim in stuck_check {
            let res = prim.resolve_col(&entity_prim_stuck);
            if let Some(..) = res {
                warn!("entity is stuck!");
                entity_prim.move_by(&Vec3::new(0.0, 0.0, BLOCK_SIZE_PLUS_SMALL));
                break;
            }
        }

        let chunk = entity_prim
            .col_center()
            .map(|e| e as i64)
            .map(|e| e.div_euc(chunk_size));
        let chunk_exists = chunk_mgr.loaded(Vec2::new(chunk.x, chunk.y));
        if !chunk_exists {
            *entity.vel_mut() = Vec3::broadcast(0.0);
            continue; //skip applying
        }

        // apply
        *entity.pos_mut() = entity_prim.col_center() - ENTITY_MIDDLE_OFFSET;
    }
}
