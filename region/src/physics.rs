// Standard
use std::sync::{RwLock};
use std::collections::HashMap;

// Library
use coord::prelude::*;

// Project
use common::{Uid};
use collision::{Collidable, CollisionResolution, Collider};

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

        // auto jump
        let auto_jump_col = Collidable::new_cuboid(middle + *entity.ctrl_vel() * 0.5 + vec3!(0.0, 0.0, 0.2), radius);
        let auto_jump = chunk_mgr.get_nearby(&auto_jump_col);
        let mut would_collide = false;
        for col in auto_jump {
            let res = col.resolve_col(&auto_jump_col);
            //if let Some(CollisionResolution::Overlap{..}) = res {
            if let Some(..) = res {
                would_collide = true;
                break;
            }
        }
        println!("would1: {}", would_collide);
        if would_collide {
            let auto_jump_col = Collidable::new_cuboid(middle + *entity.ctrl_vel() * 0.5 + vec3!(0.0, 0.0, 1.2), radius);
            let auto_jump = chunk_mgr.get_nearby(&auto_jump_col);
            let mut would_collide_afterjump = false;
            for col in auto_jump {
                let res = col.resolve_col(&auto_jump_col);
                if let Some(..) = res {
                    would_collide_afterjump = true;
                    break;
                }
            }
            println!("would2: {}", would_collide_afterjump);
            if !would_collide_afterjump {
                entity.vel_mut().z = 0.5;
            }
        }

        let speed = (*entity.vel() + *entity.ctrl_vel()) * dt;
        println!("speed: {}", speed);

        let half_chunk_scale = vec3!(0.45, 0.45, 0.45); // to forbid glitching when really fast

        let mut speed_step_cnt = 1.0;
        if speed.x.abs() / half_chunk_scale.x > speed_step_cnt {
            speed_step_cnt = speed.x.abs() / half_chunk_scale.x;
        }
        if speed.y.abs() / half_chunk_scale.y > speed_step_cnt {
            speed_step_cnt = speed.y.abs() / half_chunk_scale.y;
        }
        if speed.z.abs() / half_chunk_scale.z > speed_step_cnt {
            speed_step_cnt = speed.z.abs() / half_chunk_scale.z;
        }

        let speed_step_cnt = speed_step_cnt.ceil();
        let step = speed / speed_step_cnt;
        // execute the movement in steps of 1/2 of chunk_scale to be sure not to mess up if moving fast
        let speed_step_cnt = speed_step_cnt as i64;
        println!("speed_step_cnt: {} step: {}", speed_step_cnt, step);

        //apply movement in steps to detect glitching due to fast speed
        for _ in 0..speed_step_cnt {
            // work on new coordinates
            match &mut entity_col {
                Collidable::Cuboid { ref mut cuboid } => {
                    *cuboid.middle_mut() += step;
                }
            }

            // collision with terrain
            //TODO: evaluate to add speed to get_nerby function and just call it once
            let totest = chunk_mgr.get_nearby(&entity_col);
            println!("test agains: {:?}", totest.len());

            for col in totest {
                //println!("col {:?}", col);
                let res = col.resolve_col(&entity_col);
                if let Some(res) = res {
                    println!("res {:?}", res);
                    //apply correction
                    match res {
                        CollisionResolution::Touch{..} => {
                            //println!("touch to much");
                        },
                        CollisionResolution::Overlap{ correction, .. } => {
                            match &mut entity_col {
                                Collidable::Cuboid { ref mut cuboid } => {
                                    *cuboid.middle_mut() = *cuboid.middle() + correction;
                                }
                            }
                            // instant stop if hit anything
                            println!("correction {}", correction);
                            println!("before vel {}", entity.vel());
                            if correction.x != 0.0 {
                                entity.vel_mut().x = 0.0;
                            }
                            if correction.y != 0.0 {
                                entity.vel_mut().y = 0.0;
                            }
                            if correction.z != 0.0 {
                                entity.vel_mut().z = 0.0;
                            }
                            println!("after vel {}", entity.vel());
                        }
                    }
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
                *entity.pos_mut() = (*cuboid.middle() - vec3!(0.0, 0.0, 0.9));
            }
        }

    }
}
