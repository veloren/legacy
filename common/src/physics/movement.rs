// Library
use vek::*;

// Project
use crate::{
    physics::collision::{Primitive, ResolutionTti, PLANCK_LENGTH},
    Uid,
};

#[derive(PartialEq, Debug, Clone)]
pub struct Moveable {
    pub id: Uid, // to keep the relation between Moveable and Entity
    pub primitive: Primitive,
    pub mass: f32, //infinite mass means it doesnt move
    pub old_velocity: Vec3<f32>,
    pub velocity: Vec3<f32>,
}

impl Moveable {
    pub fn new(id: Uid, primitive: Primitive, mass: f32) -> Moveable {
        Moveable {
            id,
            primitive,
            mass,
            old_velocity: Vec3::zero(),
            velocity: Vec3::zero(),
        }
    }
}

fn handle_res(r: Option<ResolutionTti>, tti: &mut f32, normal: &mut Vec3<f32>) {
    if let Some(r) = r {
        if let ResolutionTti::WillCollide {
            tti: ltti,
            normal: lnormal,
        } = r
        {
            if ltti <= *tti {
                //debug!("colliding in tti: {}, normal {}", ltti, lnormal);
                if lnormal.magnitude() < normal.magnitude() || normal.magnitude() < 0.1 || ltti < *tti {
                    // when tti is same but we have less normal we switch
                    //info!("set normal to: {}", lnormal);
                    // if there is a collission with 2 and one with 1 block we first solve the single one
                    *normal = lnormal;
                }
                *tti = ltti;
            }
        }
    }
}

pub fn movement_tick(
    primitives: &mut Vec<(Moveable, Vec<Primitive>)>, // This function will check every colidable against all other colidable and against their own Vector of primitives
    old_primitives: &Vec<Moveable>,
    dt: f32,
) {
    for (c, local) in primitives.iter_mut() {
        c.velocity = c.old_velocity;
        let mut length = c.velocity * dt;

        // movement can be executed in max 3 steps because we are using TTI
        for _ in 0..3 {
            if length.magnitude() < PLANCK_LENGTH {
                break;
            }

            // collision with terrain
            let mut tti = 1.0; // 1.0 = full tick
            let mut normal = Vec3::new(0.0, 0.0, 0.0);

            for prim in local.iter() {
                let r = prim.time_to_impact(&c.primitive, &length);
                handle_res(r, &mut tti, &mut normal);
            }

            for op in old_primitives.iter() {
                if op.id == c.id {
                    continue;
                }
                let length = length - op.velocity * dt; // add op vel to calculate the relative velocity between both
                let r = op.primitive.time_to_impact(&c.primitive, &length);
                handle_res(r, &mut tti, &mut normal);
            }

            if tti > 0.0 {
                let movement = length * tti;
                if tti < 1.0 {
                    info!("total valid tti: {}", tti);
                    debug!("move by: {}", movement);
                }
                c.primitive.move_by(&movement);
                length -= movement;
            }

            if normal.x != 0.0 {
                debug!("full stop x");
                length.x = 0.0;
                c.velocity.x = 0.0;
            }
            if normal.y != 0.0 {
                debug!("full stop y");
                length.y = 0.0;
                c.velocity.y = 0.0;
            }
            if normal.z != 0.0 {
                //debug!("full stop z");
                length.z = 0.0;
                c.velocity.z = 0.0;
            }
        }
    }
}
