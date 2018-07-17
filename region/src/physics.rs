// Standard
use std::sync::{RwLock};
use std::collections::HashMap;

// Library
use coord::prelude::*;

// Project
use common::{Uid};
use collision::{Primitive, Collider, PLANCK_LENGTH, ResolutionTti};

// Local
use super::{Entity, VolMgr, VolState, Chunk};

pub fn tick<P: Send + Sync + 'static>(entities: &RwLock<HashMap<Uid, Entity>>,
            chunk_mgr: &VolMgr<Chunk, P>,
            chunk_size: i64,
            dt: f32) {
    let mut entities = entities.write().unwrap();
    for (.., entity) in entities.iter_mut() {
        let chunk = entity
            .pos()
            .map(|e| e as i64)
            .div_euc(vec3!([chunk_size; 3]));

        // Gravity
        let chunkobj = chunk_mgr.at(vec2!(chunk.x, chunk.y));
        if let Some(lock) = chunkobj {
            if let VolState::Exists(_,_) = *lock.read().unwrap() {
                entity.vel_mut().z -= 0.15;
            }
        }

        let middle = *entity.pos() + vec3!(0.0, 0.0, 0.9);
        let radius = vec3!(0.45, 0.45, 0.9);

        let mut entity_col = Primitive::new_cuboid(middle, radius);

        // block hopping
        let mut auto_jump_col = Primitive::new_cuboid(middle + *entity.ctrl_vel() * 0.4 + vec3!(0.0, 0.0, 0.1), radius);
        let auto_jump = chunk_mgr.get_nearby(&auto_jump_col);
        'outer: for col in auto_jump {
            let res = col.resolve_col(&auto_jump_col);
            if let Some(..) = res {
                auto_jump_col.move_by(&vec3!(0.0, 0.0, 1.0));
                let auto_jump = chunk_mgr.get_nearby(&auto_jump_col);
                for col in auto_jump {
                    let res = col.resolve_col(&auto_jump_col);
                    if let Some(..) = res {
                        break 'outer;
                    }
                }
                //TODO: Disabled for testing
                //entity.vel_mut().z = 0.55;
                break 'outer;
            }
        }

        let mut velocity = (*entity.vel() + *entity.ctrl_vel()) * dt;
        debug!("velocity: {}", velocity);
        // movement can be executed in 3 steps because we are using TTI

        // store all corrections in a vector. store the highest absolute value in every direction.
        // store how often a correction appears per absolute value
        // after all calculation. start calculating the movement
        // if corrections in all directions appear, we are stuck
        // cancel out directions
        // then apply the higest correction for one of the directions (or all remaining)

        //apply movement in steps to detect glitching due to fast speed
        for i in 0..3 {
            //TODO: undo this hacky cheat
            let fakk = vec3!(if i == 0 {1.0} else {0.0}, if i == 1 {1.0} else {0.0}, if i == 2 {1.0} else {0.0});
            let localspeed = velocity * fakk;

            println!("--- {}", localspeed);
            let mut positive_correction_max = vec3!(0.0, 0.0, 0.0);
            let mut negative_correction_max = vec3!(0.0, 0.0, 0.0);
            let mut positive_correction_cnt = vec3!(0, 0, 0);
            let mut negative_correction_cnt = vec3!(0, 0, 0);

            //entity_col.move_by(&vel_step);

            // collision with terrain
            //TODO: evaluate to add speed to get_nerby function and just call it once

            //TODO: add movement here
            let totest = chunk_mgr.get_nearby(&entity_col);
            let mut tti = 1.0;
            let mut normal = vec3!(0.0, 0.0, 0.0);

            for col in totest {
                let r = col.time_to_impact(&entity_col, &localspeed);
                if let Some(r) = r {
                    info!("colliding in tti: {:?}", r);
                    if let ResolutionTti::WillColide{tti: ltti, normal: lnormal} = r {
                        if ltti < tti {
                            warn!("colliding in tti: {}", ltti);
                            tti = ltti;
                            normal = lnormal;
                        }
                    }
                }
            }

            if tti != 1.0 {
                error!("total valid tti: {}", tti);
            } else {
                info!("total valid tti: {}", tti);
            }
            if tti > 0.0 {
                let movement = localspeed * tti;
                println!("velocity: {}", localspeed);
                println!("move by: {}", movement);
                entity_col.move_by(&movement);
                velocity -= movement;
                println!("after move: {:?}", entity_col);
            }
            println!("normal: {:?}", normal);
            if normal.length() > 1.0 {
                if normal.z != 0.0 {
                    println!("full stop z");
                    velocity.z = 0.0;
                    entity.vel_mut().z = 0.0;
                }
            } else {
                if normal.x != 0.0 {
                    println!("full stop x");
                    velocity.x = 0.0;
                    entity.vel_mut().x = 0.0;
                }
                if normal.y != 0.0 {
                    println!("full stop y");
                    velocity.y = 0.0;
                    entity.vel_mut().y = 0.0;
                }
                if normal.z != 0.0 {
                    println!("full stop z");
                    velocity.z = 0.0;
                    entity.vel_mut().z = 0.0;
                }
            }
        }

        //Friction
        *entity.vel_mut() *= 0.95_f32.powf(dt);

        // apply
        *entity.pos_mut() = entity_col.col_center() - Vec3::new(0.0, 0.0, 0.9);
    }
}
