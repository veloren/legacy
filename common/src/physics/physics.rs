// Standard
use std::{f64::consts::E, sync::Arc, time::Duration};

// Library
use parking_lot::RwLock;
use vek::*;

// Project
use crate::{
    physics::{
        collision::{Primitive, ResolutionTti, PLANCK_LENGTH},
        movement::{movement_tick, Moveable},
    },
    terrain::{VoxAbs, Voxel},
};

use crate::Uid;

// Local
use crate::terrain::{ChunkMgr, Entity};

pub const LENGTH_OF_BLOCK: f32 = 0.3;
const GROUND_GRAVITY: f32 = -9.81;
const BLOCK_SIZE_PLUS_SMALL: f32 = 1.0 + PLANCK_LENGTH;
const BLOCK_HOP_SPEED: f32 = 13.0;

fn adjust_box(low: &mut Vec3<f32>, high: &mut Vec3<f32>, dir: Vec3<f32>) {
    *low = low.map2(dir, |l, n| if n < 0.0 { l + n } else { l });
    *high = high.map2(dir, |h, n| if n > 0.0 { h + n } else { h });
}

fn adjust_primitive(col: &Primitive, low: &mut Vec3<f32>, high: &mut Vec3<f32>) {
    // get the entity boundrieds and convert them to blocks, then caluclate the entity velocity and adjust it
    // then move the playr up by BLOCK_SIZE_PLUS_SMALL for block hopping
    let abc = col.col_approx_abc().map(|e| e / 1.0);
    *low = low.map2(abc, |l, a| l - a);
    *high = high.map2(abc, |h, a| h + a);
}

// estimates the blocks around a entity that are needed during physics calculation.
fn get_nearby(col: &Primitive, velocities: &[Vec3<f32>]) -> (/*low:*/ Vec3<VoxAbs>, /*high:*/ Vec3<VoxAbs>) {
    let center = col.col_center();
    let mut low = center;
    let mut high = center;
    for v in velocities.iter() {
        adjust_box(&mut low, &mut high, *v);
    }
    adjust_primitive(col, &mut low, &mut high);

    // Workaround for fall through
    let low = low.map(|e| e.ceil() as VoxAbs - 1 - 1);
    let high = high.map(|e| e.trunc() as VoxAbs + 1 + 1);

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
    entities2: I,
    chunk_mgr: &ChunkMgr<CP>,
    dt: Duration,
) {
    const ENTITY_MIDDLE_OFFSET: Vec3<f32> = Vec3 { x: 0.0, y: 0.0, z: 0.9 };
    const ENTITY_RADIUS: Vec3<f32> = Vec3 {
        x: 0.45,
        y: 0.45,
        z: 0.9,
    };
    const ENTITY_ACC: Vec3<f32> = Vec3 {
        x: 24.0 / LENGTH_OF_BLOCK,
        y: 24.0 / LENGTH_OF_BLOCK,
        z: 28.0 / LENGTH_OF_BLOCK,
    };
    const SMALLER_THAN_BLOCK_GOING_DOWN: Vec3<f32> = Vec3 {
        x: 0.0,
        y: 0.0,
        z: -0.1,
    };
    const CONTROL_IN_AIR: Vec3<f32> = Vec3 {
        x: 0.17,
        y: 0.17,
        z: 0.0,
    };
    const CONTROL_IN_WATER: Vec3<f32> = Vec3 {
        x: 0.05,
        y: 0.05,
        z: 0.09,
    };
    const FRICTION_ON_GROUND: Vec3<f32> = Vec3 {
        x: 0.0015,
        y: 0.0015,
        z: 0.0015,
    };
    const FRICTION_IN_AIR: Vec3<f32> = Vec3 {
        x: 0.2,
        y: 0.2,
        z: 0.95,
    };
    const FRICTION_IN_WATER: Vec3<f32> = Vec3 {
        x: 0.60,
        y: 0.60,
        z: 0.30,
    };

    let dt = dt.as_float_secs() as f32;
    let mut primitives = Vec::new(); // This function will check every colidable against all other colidable and against their own Vector of primitives
    let mut old_primitives = Vec::new();

    for (id, entity) in entities {
        let entity = entity.read();

        let middle = *entity.pos() + ENTITY_MIDDLE_OFFSET;
        let entity_prim = Primitive::new_cuboid(middle, ENTITY_RADIUS);

        let mut wanted_ctrl_acc = *entity.ctrl_acc();
        let wanted_ctrl_acc_length = Vec3::new(wanted_ctrl_acc.x, wanted_ctrl_acc.y, 0.0).magnitude();
        if wanted_ctrl_acc_length > 1.0 {
            wanted_ctrl_acc.x /= wanted_ctrl_acc_length;
            wanted_ctrl_acc.y /= wanted_ctrl_acc_length;
        }
        wanted_ctrl_acc *= ENTITY_ACC;
        let wanted_offs_vel = wanted_ctrl_acc * dt;
        let wanted_vel = *entity.vel() + wanted_offs_vel;

        let gravity = Vec3::new(0.0,0.0,GROUND_GRAVITY/(1.0+E.powf(middle.z as f64 / 120.0/*adjust this to make gravity last longer in the upper areas*/-3.5/*constant move 1/(1+e^x) to the 1-0 range*/) as f32) / LENGTH_OF_BLOCK );
        let velocities = [wanted_vel, gravity * dt];

        let (low, high) = get_nearby(&entity_prim, &velocities);
        let volsample = chunk_mgr.try_get_sample(low, high);
        if let Err(_) = volsample {
            continue; //skip this entity, because not all chunks are loaded
        }
        let volsample = volsample.unwrap();
        let mut nearby_primitives = Vec::new();
        for (pos, b) in volsample.iter() {
            if b.is_solid() {
                let entity = Primitive::new_cuboid(
                    pos.map(|e| e as f32) + Vec3::new(0.5, 0.5, 0.5),
                    Vec3::new(0.5, 0.5, 0.5),
                );
                nearby_primitives.push(entity);
            }
        }

        let mut nearby_primitives_fluid = Vec::new();
        for (pos, b) in volsample.iter() {
            if b.is_fluid() {
                let entity = Primitive::new_cuboid(
                    pos.map(|e| e as f32) + Vec3::new(0.5, 0.5, 0.5),
                    Vec3::new(0.5, 0.5, 0.5),
                );
                nearby_primitives_fluid.push(entity);
            }
        }

        // is standing on ground to jump
        let mut on_ground = false;
        for prim in &nearby_primitives {
            let res = prim.time_to_impact(&entity_prim, &SMALLER_THAN_BLOCK_GOING_DOWN);
            if let Some(ResolutionTti::WillCollide { tti, .. }) = res {
                if tti < PLANCK_LENGTH * 2.0 {
                    // something really small
                    on_ground = true;
                    break;
                }
            }
        }

        // is standing in water
        let mut in_water = false;
        for prim in &nearby_primitives_fluid {
            let mut entity_prim_water = entity_prim.clone();
            entity_prim_water.move_by(&Vec3::new(0.0, 0.0, 1.0));
            let res = prim.time_to_impact(&entity_prim_water, &SMALLER_THAN_BLOCK_GOING_DOWN);
            if let Some(ResolutionTti::Overlapping { .. }) = res {
                in_water = true;
                break;
            }
        }

        //adjust movement
        let mut vel = *entity.vel()
            + if in_water {
                gravity * 0.1
            } else {
                gravity
            } * dt
            + if in_water {
                wanted_offs_vel * CONTROL_IN_WATER
            } else if on_ground {
                // calculate jump in vel not acc! assume 0.2 sec jump time
                Vec3::new(wanted_ctrl_acc.x * dt, wanted_ctrl_acc.y * dt, wanted_ctrl_acc.z * 0.2)
            } else {
                wanted_offs_vel * CONTROL_IN_AIR
            };
        vel *= (if in_water { FRICTION_IN_WATER } else if on_ground { FRICTION_ON_GROUND } else { FRICTION_IN_AIR }).map(|e| e.powf(dt));

        let mut movable = Moveable::new(*id, entity_prim, 80.0);
        movable.old_velocity = vel;
        old_primitives.push(movable.clone());
        primitives.push((movable, nearby_primitives));
    }

    movement_tick(&mut primitives, &old_primitives, dt);

    for (id, entity) in entities2 {
        for (mov, local) in primitives.iter_mut() {
            if mov.id == *id {
                // am i stuck check
                let mut entity_prim_stuck = mov.primitive.clone();
                entity_prim_stuck.scale_by(0.9);
                for prim in local.iter() {
                    let res = prim.resolve_col(&entity_prim_stuck);
                    if let Some(..) = res {
                        warn!("entity is stuck!");
                        mov.primitive.move_by(&Vec3::new(0.0, 0.0, BLOCK_SIZE_PLUS_SMALL));
                        break;
                    }
                }

                if mov.velocity.x != mov.old_velocity.x || mov.velocity.y != mov.old_velocity.y {
                    // something got stoped, try block hopping
                    let mut hopmov = mov.clone();
                    hopmov.primitive.move_by(&Vec3::new(0.0, 0.0, BLOCK_SIZE_PLUS_SMALL));
                    let mut mov_vec = vec![(hopmov, local.clone())];
                    movement_tick(&mut mov_vec, &old_primitives, dt);
                    let hopmov = &mov_vec[0].0;
                    if (hopmov.velocity.x != mov.velocity.x || hopmov.velocity.y != mov.velocity.y)
                        && (hopmov.velocity.x != 0.0 || hopmov.velocity.y != 0.0)
                    {
                        //println!("vel diff {} and {}", hopmov.velocity,  mov.velocity);
                        //mov.primitive = hopmov.primitive.clone();
                        let up = (BLOCK_HOP_SPEED * dt).min(BLOCK_SIZE_PLUS_SMALL);
                        mov.primitive.move_by(&Vec3::new(0.0, 0.0, up));
                        mov.velocity = hopmov.velocity;
                        mov.velocity.z = 0.0;
                    }
                }

                let mut entity = entity.write();
                *entity.pos_mut() = mov.primitive.col_center() - ENTITY_MIDDLE_OFFSET;
                *entity.vel_mut() = mov.velocity;

                break;
            }
        }
    }
}
