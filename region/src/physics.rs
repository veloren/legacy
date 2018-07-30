// Standard
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::clone::Clone;

// Library
use coord::prelude::*;

// Project
use common::{Uid};
use collision::{Primitive, Collider, PLANCK_LENGTH, ResolutionTti};

// Local
use super::{Entity, VolMgr, VolState, Chunk};

pub const LENGTH_OF_BLOCK : f32 = 0.3;

pub fn tick<'a, P: Send + Sync + 'static, I: Iterator<Item = (&'a Uid, &'a Arc<RwLock<Entity>>)>>(entities: I,
            chunk_mgr: &VolMgr<Chunk, P>,
            chunk_size: i64,
            dt: f32) {
    for (.., entity) in entities {
        let mut entity = entity.write().unwrap();
        let chunk = entity
            .pos()
            .map(|e| e as i64)
            .div_euc(vec3!([chunk_size; 3]));

        // Gravity
        let mut gravity_acc = vec3!(0.0, 0.0, 0.0);
        let chunkobj = chunk_mgr.at(vec2!(chunk.x, chunk.y));
        if let Some(lock) = chunkobj {
            if let VolState::Exists(_,_) = *lock.read().unwrap() {
                gravity_acc.z = -9.81 / LENGTH_OF_BLOCK;
            }
        }

        let middle = *entity.pos() + vec3!(0.0, 0.0, 0.9);
        let radius = vec3!(0.45, 0.45, 0.9);

        let mut entity_col = Primitive::new_cuboid(middle, radius);

        // is standing on ground to jump
        let mut on_ground = false;
        let can_jump_col = Primitive::new_cuboid(middle, radius);
        let auto_jump = chunk_mgr.get_nearby(&can_jump_col);
        for col in auto_jump {
            let res = col.time_to_impact(&can_jump_col, &vec3!(0.0, 0.0, -0.1));
            if let Some(ResolutionTti::WillColide{tti, ..}) = res {
                if tti < PLANCK_LENGTH*2.0 { // something really small
                    on_ground = true;
                    break;
                }
            }
        }

        let mut wanted_ctrl_acc = *entity.ctrl_acc();
        // apply checking if player can conrol (touches ground) out this in client instead of physics
        if !on_ground {
            wanted_ctrl_acc.x *= 0.2;
            wanted_ctrl_acc.y *= 0.2;
            wanted_ctrl_acc.z = 0.0;
        }

        let wanted_ctrl_acc_length = wanted_ctrl_acc.length();
        if wanted_ctrl_acc_length > 1.0 {
            wanted_ctrl_acc /= wanted_ctrl_acc_length;
        }

        // multiply by entity speed
        wanted_ctrl_acc *= vec3!(13.0 / LENGTH_OF_BLOCK, 13.0 / LENGTH_OF_BLOCK, 155.0 / LENGTH_OF_BLOCK);

        // calc acc
        let mut acc = wanted_ctrl_acc + gravity_acc;
        //println!("acc {}" , acc);

        // apply friction to acc
        if on_ground {
            acc *= 0.6_f32.powf(dt);
        } else {
            acc *= 0.95_f32.powf(dt);
        }

        // apply acc to vel
        *entity.vel_mut() += acc * dt;

        // apply friction to vel
        if on_ground {
            *entity.vel_mut() *= 0.02_f32.powf(dt);
        } else {
            *entity.vel_mut() *= 0.7_f32.powf(dt);
        }

        let mut velocity = *entity.vel() * dt;
        debug!("velocity: {}", velocity);

        // movement can be executed in max 3 steps because we are using TTI
        for _ in 0..3 {
            if velocity.length() < PLANCK_LENGTH {
                break;
            }

            // collision with terrain
            let totest = chunk_mgr.get_nearby(&entity_col);
            let mut tti = 1.0;
            let mut normal = vec3!(0.0, 0.0, 0.0);

            for col in totest {
                let r = col.time_to_impact(&entity_col, &velocity);
                if let Some(r) = r {
                    //info!("colliding in tti: {:?}", r);
                    if let ResolutionTti::WillColide{tti: ltti, normal: lnormal} = r {
                        if ltti <= tti {
                            //warn!("colliding in tti: {}, normal {}", ltti, lnormal);
                            if lnormal.length() < normal.length() || normal.length() < 0.1 || ltti < tti { // when tti is same but we have less normal we switch
                                //warn!("set normal to: {}", lnormal);
                                // if there is a collission with 2 and one with 1 block we first solfe the single one
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
                    //println!("velocity: {}", velocity);
                    debug!("move by: {}", movement);
                }
                entity_col.move_by(&movement);
                velocity -= movement;
                //println!("after move: {:?}", entity_col);
            }
            //println!("normal: {:?}", normal);

            if normal.x != 0.0 || normal.y != 0.0 {
                // block hopping
                let mut auto_jump_col = entity_col.clone();
                auto_jump_col.move_by(&vec3!(0.0, 0.0, 1.01));
                let auto_jump = chunk_mgr.get_nearby(&auto_jump_col);
                let mut collision_after_hopp = false;
                for col in auto_jump {
                    let res = col.resolve_col(&auto_jump_col);
                    if let Some(..) = res {
                        collision_after_hopp = true;
                        break;
                    }
                }
                if collision_after_hopp {
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
                    entity_col.move_by(&vec3!(0.0, 0.0, 1.01));
                }
            }
            if normal.z != 0.0 {
                debug!("full stop z");
                velocity.z = 0.0;
                entity.vel_mut().z = 0.0;
            }
        }
        // apply
        *entity.pos_mut() = entity_col.col_center() - Vec3::new(0.0, 0.0, 0.9);
    }
}
