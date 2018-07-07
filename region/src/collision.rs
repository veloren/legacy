use std::cmp;
use coord::prelude::*;
use {Volume, Voxel, Cell};

#[derive(PartialEq, Debug)]
pub struct Cuboid {
    middle: Vec3<f64>,
    radius: Vec3<f64>,
}

#[derive(PartialEq, Debug)]
pub enum CollisionResolution {
    Touch { point: Vec3<f64> },
    Overlap { point: Vec3<f64>, correction: Vec3<f64>}, //correction = movement of the second parameter to touch the first parameter
}

#[derive(PartialEq, Debug)]
pub enum Collidable {
    Cuboid { cuboid: Cuboid },
    //add more here
}

pub fn resolve_collision(a: &Collidable, b: &Collidable) -> Option<CollisionResolution> {
    match a {
        Collidable::Cuboid { cuboid: a } => {
            match b {
                Collidable::Cuboid { cuboid: b } => {
                    cuboid_cuboid_col(a,b)
                },
            }
        },
    }
}

impl Cuboid {
    pub fn new(middle: Vec3<f64>, radius: Vec3<f64>) -> Self {
        Cuboid {
            middle,
            radius,
        }
    }

    pub fn lower(&self) -> Vec3<f64> {
        self.middle - self.radius
    }

    pub fn upper(&self) -> Vec3<f64> {
        self.middle + self.radius
    }

    pub fn middle(&self) -> &Vec3<f64> { &self.middle }
    pub fn middle_mut(&mut self) -> &mut Vec3<f64> { &mut self.middle }
    pub fn radius(&self) -> &Vec3<f64> { &self.radius }
    pub fn radius_mut(&mut self) -> &mut Vec3<f64> { &mut self.radius }
}

fn cuboid_cuboid_col(a: &Cuboid, b: &Cuboid) -> Option<CollisionResolution> {
    let la = a.lower();
    let ua = a.upper();
    let lb = b.lower();
    let ub = b.upper();
    if ua.x >= lb.x && la.x <= ub.x &&
       ua.y >= lb.y && la.y <= ub.y &&
       ua.z >= lb.z && la.z <= ub.z {
              //collide or touch
              let moved = *b.middle() - *a.middle();
              let abs_moved = vec3!(moved.x.abs(), moved.y.abs(), moved.z.abs());
              let border_diff = *a.radius() - abs_moved;
              let signed_diff_to_border;
              let signed_relevant_b_radius;
              // test which is nearest
              if border_diff.x <= border_diff.y && border_diff.x <= border_diff.z {
                  //x
                  if b.middle().x < a.middle().x {
                      signed_diff_to_border = vec3!(-border_diff.x, 0.0, 0.0);
                      signed_relevant_b_radius = vec3!(-b.radius().x, 0.0, 0.0);
                  } else {
                      signed_diff_to_border = vec3!(border_diff.x, 0.0, 0.0);
                      signed_relevant_b_radius = vec3!(b.radius().x, 0.0, 0.0);
                  }
              } else if border_diff.y <= border_diff.x && border_diff.y <= border_diff.z {
                   //y
                   if b.middle().y < a.middle().y {
                       signed_diff_to_border = vec3!(0.0, -border_diff.y, 0.0);
                       signed_relevant_b_radius = vec3!(0.0, -b.radius().y, 0.0);
                   } else {
                       signed_diff_to_border = vec3!(0.0, border_diff.y, 0.0);
                       signed_relevant_b_radius = vec3!(0.0, b.radius().y, 0.0);
                   }
               } else {
                   if !(border_diff.z <= border_diff.x && border_diff.z <= border_diff.y) {
                        println!("border_diff: {}", border_diff);
                        assert!(false);
                   }

                   //z
                   if b.middle().z < a.middle().z {
                       signed_diff_to_border = vec3!(0.0, 0.0, -border_diff.z);
                       signed_relevant_b_radius = vec3!(0.0, 0.0, -b.radius().z);
                   } else {
                       signed_diff_to_border = vec3!(0.0, 0.0, border_diff.z);
                       signed_relevant_b_radius = vec3!(0.0, 0.0, b.radius().z);
                   }
               }

              let point = *b.middle() + signed_diff_to_border;
              let correction = signed_diff_to_border + signed_relevant_b_radius;

              //println!("point {}, correction {}, signed_diff_to_border {}, relevant_a_radius {}", point, correction, signed_diff_to_border, signed_relevant_b_radius);

              if (correction == vec3!(0.0, 0.0, 0.0)) {
                  assert!( !(ua.x > lb.x && la.x < ub.x &&
                             ua.y > lb.y && la.y < ub.y &&
                             ua.z > lb.z && la.z < ub.z));
                  return Some(CollisionResolution::Touch{
                      point,
                  });
              } else {
                  return Some(CollisionResolution::Overlap{
                      point,
                      correction,
                  });
              }
        };
    None
}
