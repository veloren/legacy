use coord::prelude::*;

#[derive(PartialEq, Debug)]
pub struct Cuboid {
    middle: Vec3<f32>,
    radius: Vec3<f32>,
}

#[derive(PartialEq, Debug)]
pub enum CollisionResolution {
    Touch { point: Vec3<f32> },
    Overlap { point: Vec3<f32>, correction: Vec3<f32>},
}

#[derive(PartialEq, Debug)]
pub enum Collidable {
    Cuboid { cuboid: Cuboid },
    //add more here
}

pub trait Collider {
    fn get_nearby(&self, pos: Vec3<f32>, radius: Vec3<f32>) -> Vec<Collidable>;
}

impl Collidable {
    // CollisionResolution is the minimal movement of b to avoid overlap, but allow touch with self
    pub fn resolve_col(&self, b: &Collidable) -> Option<CollisionResolution> {
        match self {
            Collidable::Cuboid { cuboid: a } => {
                match b {
                    Collidable::Cuboid { cuboid: b } => {
                        cuboid_cuboid_col(a,b)
                    },
                }
            },
        }
    }

    pub fn center_of_mass(&self) -> Vec3<f32> {
        match self {
            Collidable::Cuboid { cuboid: a } => a.middle,
        }
    }

    // when using the collision center, the outer_aproximation_sphere can be minimal
    // implement it fast!
    pub fn col_center(&self) -> Vec3<f32> {
        match self {
            Collidable::Cuboid { cuboid: a } => a.middle,
        }
    }

    // Collidable musst fully fit into a Sphere with the middle col_center and the radius col_aprox_rad
    // implement it fast!

    //actually is no radius, its x,y,z components of a Vector
    //TODO: need performant refactor, or * SQRT(3)
    pub fn col_aprox_rad(&self) -> Vec3<f32> {
        match self {
            Collidable::Cuboid { cuboid: a } => a.radius,
        }
    }
}

impl Collidable {
    pub fn new_cuboid(middle: Vec3<f32>, radius: Vec3<f32>) -> Self {
        Collidable::Cuboid{ cuboid: Cuboid::new(middle, radius) }
    }
}

impl Cuboid {
    pub fn new(middle: Vec3<f32>, radius: Vec3<f32>) -> Self {
        Cuboid {
            middle,
            radius,
        }
    }

    #[allow(dead_code)] pub fn lower(&self) -> Vec3<f32> {
        self.middle - self.radius
    }

    #[allow(dead_code)] pub fn upper(&self) -> Vec3<f32> {
        self.middle + self.radius
    }

    #[allow(dead_code)] pub fn middle(&self) -> &Vec3<f32> { &self.middle }
    #[allow(dead_code)] pub fn middle_mut(&mut self) -> &mut Vec3<f32> { &mut self.middle }
    #[allow(dead_code)] pub fn radius(&self) -> &Vec3<f32> { &self.radius }
    #[allow(dead_code)] pub fn radius_mut(&mut self) -> &mut Vec3<f32> { &mut self.radius }
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

              if correction.length() < 0.001 { // some aproximation here
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
