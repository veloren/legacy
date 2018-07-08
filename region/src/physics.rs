// Standard
use std::sync::{RwLock};
use std::collections::HashMap;

// Library
use coord::prelude::*;

// Project
use common::{Uid};

// Local
use super::{Entity, VolMgr, VolState, Chunk, Voxel};

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
        if let Some(c) = chunk_mgr.at(vec2!(chunk.x, chunk.y)) {
            if let VolState::Exists(_, _) = *c.read().unwrap() {
                let _below_feet = *entity.pos() - vec3!(0.0, 0.0, -0.1);
                if entity
                    .get_aabb()
                    .shift_by(vec3!(0.0, 0.0, -0.1)) // Move it a little below the player to check whether we're on the ground
                    .collides_with(chunk_mgr) {
                    entity.vel_mut().z = 0.0;
                } else {
                    entity.vel_mut().z -= 0.15;
                }
            }
        }
        // Gravity
        //entity.vel_mut().z -= 0.15;

        let dpos = (*entity.vel() + *entity.ctrl_vel()) * dt;

        // Resolve collisions with the terrain
        let dpos = entity.get_aabb().resolve_with(chunk_mgr, dpos);

        *entity.pos_mut() += dpos;

        /*
        let vel = *entity.vel() + *entity.ctrl_vel();
        *entity.pos_mut() += vel * dt;
        */

        /*
        let player_col = Collidable::Cuboid{cuboid: Cuboid::new(vec3!(
            (entity.pos().x as i64 + x),
            (entity.pos().y as i64 + y),
            (entity.pos().z as i64 + z)
        ), vec3!(
            0.5, 0.5, 0.5
        ))};
        */

        /*
        for x in -1..2 {
            for y in -1..2 {
                for z in -1..2 {
                    let vox = chunk_mgr.get_voxel(vec3!(
                        (entity.pos().x as i64 + x),
                        (entity.pos().y as i64 + y),
                        (entity.pos().z as i64 + z)
                    ));
                    if vox.is_solid() {
                        let a = Collidable{}

                        ERROROROROROOROR
                        let player_col = Collidable::Cuboid{cuboid: Cuboid::new{vec3!(
                            (entity.pos().x as i64 + x),
                            (entity.pos().y as i64 + y),
                            (entity.pos().z as i64 + z)
                        ), vec3!(
                            0.5, 0.5, 0.5,
                        )}}}

                        let col_res = resolve_collision()
                        entity.move_dir_mut().z = 0.0;
                        entity.pos_mut().z += 0.0025;
                    };
                }
            }
        }
        */

        /*
        while chunk_mgr.get_voxel_at(vec3!(
            entity.pos().x as i64,
            entity.pos().y as i64,
            entity.pos().z as i64
        )).is_solid() {
            entity.vel_mut().z = 0.0;
            entity.pos_mut().z += 0.0025;
        }*/


    }
}
