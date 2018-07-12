// Standard
use std::sync::{RwLock};
use std::collections::HashMap;

// Library
use coord::prelude::*;

// Project
use common::{Uid};
use collision::{Collidable, Collider};

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

        let mut entity_col = Collidable::new_cuboid(middle, radius);

        // block hopping
        let mut auto_jump_col = Collidable::new_cuboid(middle + *entity.ctrl_vel() * 0.4 + vec3!(0.0, 0.0, 0.1), radius);
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
                entity.vel_mut().z = 0.55;
                break 'outer;
            }
        }

        let velocity = (*entity.vel() + *entity.ctrl_vel()) * dt;
        debug!("velocity: {}", velocity);

        let half_chunk_scale = vec3!(0.45, 0.45, 0.45); // to forbid glitching when really fast

        let mut speed_step_cnt = 1.0;
        //TODO: refactor with new coord
        if velocity.x.abs() / half_chunk_scale.x > speed_step_cnt {
            speed_step_cnt = velocity.x.abs() / half_chunk_scale.x;
        }
        if velocity.y.abs() / half_chunk_scale.y > speed_step_cnt {
            speed_step_cnt = velocity.y.abs() / half_chunk_scale.y;
        }
        if velocity.z.abs() / half_chunk_scale.z > speed_step_cnt {
            speed_step_cnt = velocity.z.abs() / half_chunk_scale.z;
        }

        let speed_step_cnt = speed_step_cnt.ceil();
        let vel_step = velocity / speed_step_cnt;
        // execute the movement in steps of 1/2 of chunk_scale to be sure not to mess up if moving fast
        let speed_step_cnt = speed_step_cnt as i64;
        debug!("speed_step_cnt: {} step: {}", speed_step_cnt, vel_step);

        //apply movement in steps to detect glitching due to fast speed
        for _ in 0..speed_step_cnt {
            // work on new coordinates
            entity_col.move_by(&vel_step);

            // collision with terrain
            //TODO: evaluate to add speed to get_nerby function and just call it once
            let totest = chunk_mgr.get_nearby(&entity_col);

            for col in totest {
                //debug!("col {:?}", col);
                let res = col.resolve_col(&entity_col);
                if let Some(res) = res {
                    debug!("res {:?}", res);
                    //apply correction
                    if res.is_touch() {
                        continue;
                    }
                    entity_col.move_by(&res.correction);

                    // instant stop if hit anything
                    debug!("correction {}", res.correction);
                    debug!("before vel {}", entity.vel());

                    //TODO: refactor with new coord
                    if res.correction.x != 0.0 {
                        entity.vel_mut().x = 0.0;
                    }
                    if res.correction.y != 0.0 {
                        entity.vel_mut().y = 0.0;
                    }
                    if res.correction.z != 0.0 {
                        entity.vel_mut().z = 0.0;
                    }
                    debug!("after vel {}", entity.vel());
                }
            }

            //Collision with other enteties
            //TODO: consider all movements equal: so if 2 people run in each other both can walk 1/2 the distance
            //for (.., other_entity) in entities.iter_mut() {

            //}
        }

        //Friction
        *entity.vel_mut() *= 0.95_f32.powf(dt);

        match &mut entity_col {
            Collidable::Cuboid { ref mut cuboid } => {
                *entity.pos_mut() = *cuboid.middle() - Vec3::new(0.0, 0.0, 0.9);
            }
        }

    }
}
