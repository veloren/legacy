// Standard
use std::{collections::HashMap, sync::Arc, time::Duration};

// Library
use parking_lot::RwLock;
use vek::*;

// Project
use crate::{
    physics::{
        collision::{Primitive, ResolutionTti, PLANCK_LENGTH},
        movement::{limit_entity_movement, movement_tick, MovingBody},
    },
    terrain::{VoxAbs, Voxel},
};

use crate::Uid;

// Local
use crate::terrain::{ChunkMgr, Entity};

pub const LENGTH_OF_BLOCK: f32 = 0.3;
const GROUND_GRAVITY: f32 = -9.81;
const BLOCK_SIZE_PLUS_SMALL: f32 = 1.0 + PLANCK_LENGTH;
const BLOCK_HOP_SPEED: f32 = 15.0;

fn adjust_box(low: &mut Vec3<f32>, high: &mut Vec3<f32>, dir: Vec3<f32>) {
    // if dir is lower that low adjust low so that dir fits in. Accordingly if dir is higher than high.
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
    let (mut low, mut high) = (center, center);
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
    I: Iterator<Item = (&'a Uid, &'a Arc<RwLock<Entity<EP>>>)> + Clone,
>(
    entities: I,
    chunk_mgr: &ChunkMgr<CP>,
    dt: Duration,
) {
    const ENTITY_MIDDLE_OFFSET: Vec3<f32> = Vec3 { x: 0.0, y: 0.0, z: 0.9 };
    const ENTITY_RADIUS: Vec3<f32> = Vec3 {
        x: 0.45,
        y: 0.45,
        z: 0.9,
    };
    const BLOCK_MIDDLE: Vec3<f32> = Vec3 { x: 0.5, y: 0.5, z: 0.5 };
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
    let mut moving_bodies = HashMap::new(); // This function will check every colidable against all other colidable and against their own Vector of primitives
    let mut obstacles = HashMap::new();

    for (id, entity) in entities.clone() {
        let entity = entity.read();

        let middle = *entity.pos() + ENTITY_MIDDLE_OFFSET;
        let entity_prim = Primitive::new_cuboid(middle, ENTITY_RADIUS);

        let wanted_ctrl_acc = limit_entity_movement(*entity.ctrl_acc()) * ENTITY_ACC;
        let wanted_offs_vel = wanted_ctrl_acc * dt;

        let gravity = Vec3::new(0.0, 0.0, GROUND_GRAVITY / LENGTH_OF_BLOCK);
        //let gravity = Vec3::new(0.0,0.0,GROUND_GRAVITY/(1.0+E.powf(middle.z as f64 / 120.0/*adjust this to make gravity last longer in the upper areas*/-3.5/*constant move 1/(1+e^x) to the 1-0 range*/) as f32) / LENGTH_OF_BLOCK );
        let velocities = [*entity.vel() + wanted_offs_vel, gravity * dt];

        let (low, high) = get_nearby(&entity_prim, &velocities);
        let volsample = chunk_mgr.try_get_sample(low, high);
        if let Err(_) = volsample {
            continue; //skip this entity, because not all chunks are loaded
        }
        let volsample = volsample.unwrap();
        let mut nearby_primitives = Vec::new();
        let mut nearby_primitives_fluid = Vec::new();
        for (pos, b) in volsample.iter() {
            if b.is_solid() {
                nearby_primitives.push(Primitive::new_cuboid(
                    pos.map(|e| e as f32) + BLOCK_MIDDLE,
                    BLOCK_MIDDLE,
                ));
            }
            if b.is_fluid() {
                nearby_primitives_fluid.push(Primitive::new_cuboid(
                    pos.map(|e| e as f32) + BLOCK_MIDDLE,
                    BLOCK_MIDDLE,
                ));
            }
        }

        // is standing on ground to jump
        let on_ground = nearby_primitives
            .iter()
            .find(|prim| {
                if let Some(ResolutionTti::WillCollide { tti, .. }) =
                    prim.time_to_impact(&entity_prim, &SMALLER_THAN_BLOCK_GOING_DOWN)
                {
                    tti < PLANCK_LENGTH * 2.0
                } else {
                    false
                }
            })
            .is_some();

        // is standing in water
        let mut entity_prim_water = entity_prim.clone();
        entity_prim_water.move_by(&Vec3::new(0.0, 0.0, 1.0));
        let in_water = nearby_primitives_fluid
            .iter()
            .find(|prim| {
                if let Some(ResolutionTti::Overlapping { .. }) =
                    prim.time_to_impact(&entity_prim_water, &SMALLER_THAN_BLOCK_GOING_DOWN)
                {
                    true
                } else {
                    false
                }
            })
            .is_some();

        //adjust movement
        let mut vel = *entity.vel()
            + if in_water { gravity * 0.1 } else { gravity } * dt
            + if in_water {
                wanted_offs_vel * CONTROL_IN_WATER
            } else if on_ground {
                // calculate jump in vel not acc! assume 0.2 sec jump time
                Vec3::new(wanted_ctrl_acc.x * dt, wanted_ctrl_acc.y * dt, wanted_ctrl_acc.z * 0.2)
            } else {
                wanted_offs_vel * CONTROL_IN_AIR
            };
        vel *= (if in_water {
            FRICTION_IN_WATER
        } else if on_ground {
            FRICTION_ON_GROUND
        } else {
            FRICTION_IN_AIR
        })
        .map(|e| e.powf(dt));

        let m = MovingBody {
            id: *id,
            mass: 80.0,
            primitive: entity_prim,
            velocity: vel,
        };
        moving_bodies.insert(*id, (m.clone(), nearby_primitives));
        obstacles.insert(*id, m);
    }

    movement_tick(moving_bodies.values_mut(), obstacles.values(), dt);

    for (id, entity) in entities {
        if let (Some((mov, nearby)), Some(old_mov)) = (moving_bodies.get_mut(id), obstacles.get(id)) {
            // am i stuck check
            let mut entity_prim_stuck = mov.primitive.clone();
            entity_prim_stuck.scale_by(0.9);
            for prim in nearby.iter() {
                if let Some(..) = prim.resolve_col(&entity_prim_stuck) {
                    warn!("entity is stuck!");
                    mov.primitive.move_by(&(Vec3::unit_z() * BLOCK_SIZE_PLUS_SMALL));
                    break;
                }
            }

            if mov.velocity.x != old_mov.velocity.x || mov.velocity.y != old_mov.velocity.y {
                // something got stoped, try block hopping
                let cur_percent_of_hop = (mov.primitive.col_center().z + PLANCK_LENGTH /*needs to be done before substract because of f32 percision CPU inaccurate for 128.9 - 0.9 = 127.9999 */- ENTITY_MIDDLE_OFFSET.z).fract();
                let needed_for_step = Vec3::unit_z() * (BLOCK_SIZE_PLUS_SMALL - cur_percent_of_hop + PLANCK_LENGTH);
                //check top first
                if nearby
                    .iter()
                    .find(|prim| match prim.time_to_impact(&mov.primitive, &needed_for_step) {
                        Some(ResolutionTti::WillCollide { tti, .. }) => tti < 1.0,
                        _ => false,
                    })
                    .is_none()
                {
                    let mut mov_arr = [(mov.clone(), nearby.clone())]; //TODO: remove these clones
                    mov_arr[0].0.primitive.move_by(&needed_for_step);
                    mov_arr[0].0.velocity = old_mov.velocity;

                    movement_tick(mov_arr.iter_mut(), obstacles.values(), dt);

                    let hopmov = &mov_arr[0].0;
                    if (hopmov.velocity.x != mov.velocity.x || hopmov.velocity.y != mov.velocity.y)
                        && (hopmov.velocity.x != 0.0 || hopmov.velocity.y != 0.0)
                    {
                        let up = (BLOCK_HOP_SPEED * dt).min(needed_for_step.z);
                        mov.primitive.move_by(&(Vec3::unit_z() * up));
                        mov.velocity = hopmov.velocity;
                        mov.velocity.z = 0.0;
                    }
                }
            }

            let mut entity = entity.write();
            *entity.pos_mut() = mov.primitive.col_center() - ENTITY_MIDDLE_OFFSET;
            *entity.vel_mut() = mov.velocity;
        }
    }
}
