// Library
use vek::*;

// Project
use crate::{
    physics::collision::{Primitive, ResolutionTti, PLANCK_LENGTH},
    Uid,
};

#[derive(PartialEq, Clone, Debug)]
pub struct MovingBody {
    pub id: Uid,   // to keep the relation between Moveable and Entity
    pub mass: f32, //infinite mass means it doesnt move
    pub primitive: Primitive,
    pub velocity: Vec3<f32>,
}

fn handle_res(r: Option<ResolutionTti>, tti: &mut f32, normal: &mut Vec3<f32>) {
    if let Some(ResolutionTti::WillCollide {
        tti: ltti,
        normal: lnormal,
    }) = r
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

pub fn limit_entity_movement(planed: Vec3<f32>) -> Vec3<f32> {
    let len: f32 = Vec2::from(planed).magnitude();
    let mut res = planed;
    if len > 1.0 {
        res.x /= len;
        res.y /= len;
    }
    return res;
}

// This function will check every moveable against all old_primitives and against their own Vector of primitives
// After that primitives is modified to the new state after dt

pub fn movement_tick<
    'a,
    I: Iterator<Item = &'a mut (MovingBody, Vec<Primitive>)>,
    I2: Iterator<Item = &'a MovingBody> + Clone,
>(
    to_move: I,
    obstacles: I2,
    dt: f32,
) {
    for (m, nearby) in to_move {
        let mut length = m.velocity * dt;

        // movement can be executed in max 3 steps because we are using TTI
        for _ in 0..3 {
            if length.magnitude() < PLANCK_LENGTH {
                break;
            }

            // collision with terrain
            let mut tti = 1.0; // 1.0 = full tick
            let mut normal = Vec3::new(0.0, 0.0, 0.0);

            for prim in nearby.iter() {
                let r = prim.time_to_impact(&m.primitive, &length);
                handle_res(r, &mut tti, &mut normal);
            }

            for op in obstacles.clone() {
                if op.id == m.id {
                    continue;
                }
                let length = length - op.velocity * dt; // add op vel to calculate the relative velocity between both
                let r = op.primitive.time_to_impact(&m.primitive, &length);
                handle_res(r, &mut tti, &mut normal);
            }

            if tti > 0.0 {
                let movement = length * tti;
                if tti < 1.0 {
                    //info!("total valid tti: {}", tti);
                    //debug!("move by: {}", movement);
                }
                m.primitive.move_by(&movement);
                length -= movement;
            }

            if normal.x != 0.0 {
                //debug!("full stop x");
                length.x = 0.0;
                m.velocity.x = 0.0;
            }
            if normal.y != 0.0 {
                //debug!("full stop y");
                length.y = 0.0;
                m.velocity.y = 0.0;
            }
            if normal.z != 0.0 {
                //debug!("full stop z");
                length.z = 0.0;
                m.velocity.z = 0.0;
            }
        }
    }
}
