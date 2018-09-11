// Standard
use std::{
    cmp::{Ord, Ordering},
    f32::{consts::SQRT_2, INFINITY},
};

// Library
use vek::*;

#[derive(PartialEq, Debug, Clone)]
pub struct Cuboid {
    middle: Vec3<f32>,
    radius: Vec3<f32>,
}

#[derive(PartialEq, Debug)]
pub struct ResolutionCol {
    pub center: Vec3<f32>,
    pub correction: Vec3<f32>,
}

#[derive(PartialEq, Debug)]
pub enum ResolutionTti {
    WillCollide { tti: f32, normal: Vec3<f32> }, // tti can be 0.0 when they will overlap in future, normal is facing away away from Primitive at position of impact
    Touching { normal: Vec3<f32> }, // happens if direction is another than Primitive, i will never collide but i am tucing
    Overlapping { since: f32 },
}

#[derive(PartialEq, Debug, Clone)]
pub enum Primitive {
    Cuboid { cuboid: Cuboid },
    //add more here
}

//when checking against something containing multiple Primitives, we need to implement a Collider that returns a Iterator to all Primitives to test, e.g. for the Chunks
pub trait Collider<'a> {
    type Iter: Iterator<Item = Primitive>;

    fn get_nearby(&'a self, col: &Primitive) -> Self::Iter;
    fn get_nearby_dir(&'a self, col: &Primitive, dir: Vec3<f32>) -> Self::Iter;
}

pub const PLANCK_LENGTH: f32 = 0.001; // smallest unit of meassurement in collision, no guarantees behind this point

impl ResolutionCol {
    #[allow(dead_code)]
    pub fn is_touch(&self) -> bool {
        self.correction.x < PLANCK_LENGTH && self.correction.y < PLANCK_LENGTH && self.correction.z < PLANCK_LENGTH
    }
}

impl Primitive {
    // CollisionResolution is the minimal movement of b to avoid overlap, but allow touch with self
    /*
      Collision Resolution is done the following way: we evaluate if to Primitives overlap.
      When they everlap we calculate the center of mass inside the overlapping area (currently center of mass = center)
      We then calcululate a vector beginning from the center of mass ob the overlapping area. to the border of the overlapping area.
      The directin of the fector should be directly towards the center of mass of the second Primitive.
    */
    pub fn resolve_col(&self, b: &Primitive) -> Option<ResolutionCol> {
        match self {
            Primitive::Cuboid { cuboid: a } => match b {
                Primitive::Cuboid { cuboid: b } => a.cuboid_col(b),
            },
        }
    }

    // Time to impact of b with self when b travels in dir
    /*
      Collision Resolution is done the following way: we evaluate the nearest sides between both Primitives.
      When they collide we check that the other sides are also near each other.
      If they are we know it will impact after this time.
      We choose the smallest tti.
      If it is not colliding it might happen, that it's touching and moving alogside the other Primitive
      If the TTI is negative, they we are either behind the object, or are already coliding with it.
      We need to differenciate those cases, if no collision will occur, it returns None.
    */
    pub fn time_to_impact(&self, b: &Primitive, dir: &Vec3<f32>) -> Option<ResolutionTti> {
        match self {
            Primitive::Cuboid { cuboid: a } => match b {
                Primitive::Cuboid { cuboid: b } => a.cuboid_tti(b, dir),
            },
        }
    }

    // move center of mass
    pub fn move_by(&mut self, delta: &Vec3<f32>) {
        match self {
            Primitive::Cuboid { cuboid: a } => a.middle += *delta,
        }
    }

    // scale everything to or from center of mass
    pub fn scale_by(&mut self, factor: f32) {
        match self {
            Primitive::Cuboid { cuboid: a } => a.radius *= factor,
        }
    }

    pub fn center_of_mass(&self) -> Vec3<f32> {
        match self {
            Primitive::Cuboid { cuboid: a } => a.middle,
        }
    }

    // when using the collision center, the outer_approximation_sphere can be minimal
    // implement it fast!
    #[allow(dead_code)]
    pub fn col_center(&self) -> Vec3<f32> {
        match self {
            Primitive::Cuboid { cuboid: a } => a.middle,
        }
    }

    // returns the 3 radii of a spheroid where the object fits exactly in
    // implement it fast!
    //TODO: evaluate if this is a so fast method for checking somewhere actually
    #[allow(dead_code)]
    pub fn col_approx_rad(&self) -> Vec3<f32> {
        match self {
            Primitive::Cuboid { cuboid: a } => a.radius * SQRT_2, // SQRT(2) is correct for sphere, havent it checked for an spheroid tbh
        }
    }

    // returns a cube where the object fits in exactly
    // implement it fast!
    pub fn col_approx_abc(&self) -> Vec3<f32> {
        match self {
            Primitive::Cuboid { cuboid: a } => a.radius,
        }
    }
}

impl Primitive {
    pub fn new_cuboid(middle: Vec3<f32>, radius: Vec3<f32>) -> Self {
        Primitive::Cuboid {
            cuboid: Cuboid::new(middle, radius),
        }
    }
}

impl Cuboid {
    pub fn new(middle: Vec3<f32>, radius: Vec3<f32>) -> Self { Cuboid { middle, radius } }

    fn vector_touch_border(radius: Vec3<f32>, direction: Vec3<f32>) -> Vec3<f32> {
        let first_hit = radius / direction;
        let first_hit = first_hit.map(|e| e.abs());
        let min = if first_hit.x <= first_hit.y && first_hit.x <= first_hit.z {
            first_hit.x
        } else if first_hit.y <= first_hit.x && first_hit.y <= first_hit.z {
            first_hit.y
        } else {
            first_hit.z
        };
        return direction * min;
    }

    fn cuboid_col(&self, b: &Cuboid) -> Option<ResolutionCol> {
        let a = self;
        let la = a.lower();
        let ua = a.upper();
        let lb = b.lower();
        let ub = b.upper();
        if ua.x >= lb.x && la.x <= ub.x && ua.y >= lb.y && la.y <= ub.y && ua.z >= lb.z && la.z <= ub.z {
            //collide or touch
            let col_middle = (*a.middle() + *b.middle()) / 2.0;
            let col_radius = *a.middle() - *b.middle();
            let col_radius = Vec3::new(col_radius.x.abs(), col_radius.y.abs(), col_radius.z.abs());
            let col_radius = col_radius - *a.radius() - *b.radius();

            let mut direction = *b.middle() - col_middle;
            if direction == Vec3::new(0.0, 0.0, 0.0) {
                direction = Vec3::new(0.0, 0.0, 1.0);
            }
            let force = Cuboid::vector_touch_border(col_radius, direction);
            let force = force.map(|e| if e.abs() < PLANCK_LENGTH { 0.0 } else { e }); // apply PLANCK_LENGTH to force
            return Some(ResolutionCol {
                center: col_middle,
                correction: force,
            });
        };
        None
    }

    fn cuboid_tti(&self, b: &Cuboid, dir: &Vec3<f32>) -> Option<ResolutionTti> {
        //calculate areas which collide based on dir
        // e.g. area.x is the x cordinate of the area
        let a = self;
        let a_middle_elem = a.middle.into_array();
        let b_middle_elem = b.middle.into_array();
        let a_radius_elem = a.radius.into_array();
        let b_radius_elem = b.radius.into_array();
        let mut a_area = [0.0; 3];
        let mut b_area = [0.0; 3];
        let mut normals: [Vec3<f32>; 3] = [Vec3::new(0.0, 0.0, 0.0); 3];
        let mut tti_raw: [f32; 3] = [0.0; 3];
        let mut tti: [f32; 3] = [0.0; 3];
        let mut minimal_collision_tti: [f32; 3] = [0.0; 3]; //minimal tti value which equals a collision is already happening
        let dire = dir.into_array();
        //debug("a_middle_elem {:?}; b_middle_elem {:?}", a_middle_elem, b_middle_elem);
        //needs to be calculated for every area of the cuboid, happily it's not rotated, so its just the 3 axis
        for i in 0..3 {
            if dire[i] == 0.0 {
                //area is not filled correctly in this case, we compare middle
                let midr = (a_middle_elem[i] - b_middle_elem[i]).abs();
                let perimeterr = a_radius_elem[i] + b_radius_elem[i];
                minimal_collision_tti[i] = -INFINITY;
                //debug!("midr {:?}; perimeterr {:?}", midr, perimeterr);
                tti_raw[i] = if midr + PLANCK_LENGTH > perimeterr && midr - PLANCK_LENGTH < perimeterr {
                    0.0
                } else {
                    if midr >= perimeterr {
                        INFINITY // no movement and no collsision
                    } else {
                        -INFINITY // there is a collision
                    }
                };
                if tti_raw[i].is_sign_negative() && // it detects collision, detects -INFINITY
                   midr >= (a_radius_elem[i] + b_radius_elem[i])
                {
                    // but distance is higher than radius
                    tti[i] = INFINITY; //no collision will ocur, like ever
                } else {
                    tti[i] = tti_raw[i];
                    if tti[i] > -PLANCK_LENGTH && tti[i] < PLANCK_LENGTH {
                        // PLANCK LENGTH correction
                        tti[i] = 0.0
                    }
                }
                if a_middle_elem[i] < b_middle_elem[i] {
                    normals[i] = Vec3::new(
                        if i == 0 { 1.0 } else { 0.0 },
                        if i == 1 { 1.0 } else { 0.0 },
                        if i == 2 { 1.0 } else { 0.0 }
                    );
                } else if a_middle_elem[i] > b_middle_elem[i] {
                    normals[i] = Vec3::new(
                        if i == 0 { -1.0 } else { 0.0 },
                        if i == 1 { -1.0 } else { 0.0 },
                        if i == 2 { -1.0 } else { 0.0 }
                    );
                }
            } else {
                if dire[i] < 0.0 {
                    a_area[i] = a_middle_elem[i] + a_radius_elem[i];
                    b_area[i] = b_middle_elem[i] - b_radius_elem[i];
                    normals[i] = Vec3::new(
                        if i == 0 { 1.0 } else { 0.0 },
                        if i == 1 { 1.0 } else { 0.0 },
                        if i == 2 { 1.0 } else { 0.0 }
                    );
                } else if dire[i] > 0.0 {
                    a_area[i] = a_middle_elem[i] - a_radius_elem[i];
                    b_area[i] = b_middle_elem[i] + b_radius_elem[i];
                    normals[i] = Vec3::new(
                        if i == 0 { -1.0 } else { 0.0 },
                        if i == 1 { -1.0 } else { 0.0 },
                        if i == 2 { -1.0 } else { 0.0 }
                    );
                } else {
                    panic!("we checked above that dire[i] must not be 0.0");
                }
                //debug!("a_area {:?}; b_area {:?}", a_area, b_area);
                minimal_collision_tti[i] = -(a_radius_elem[i] + b_radius_elem[i]) * 2.0 / dire[i].abs();
                tti_raw[i] = (a_area[i] - b_area[i]) / dire[i];
                if tti_raw[i].is_sign_negative() && // it detects collision, detects -INFINITY
                   (a_area[i] - b_area[i]).abs() >= (a_radius_elem[i] + b_radius_elem[i]) * 2.0
                {
                    // but distance is higher than radius
                    tti[i] = INFINITY; //no collision will ocur, like ever
                } else {
                    tti[i] = tti_raw[i];
                    if tti[i] > -PLANCK_LENGTH && tti[i] < PLANCK_LENGTH {
                        // PLANCK LENGTH correction
                        tti[i] = 0.0
                    }
                }
            }
        }
        // tti now contains a value per coordinate. pos=will collide in, 0=touches right now, negative=is colliding since, INF=will never collide

        //info!("tti_raw {:?}", tti_raw);
        //info!("tti {:?}", tti);

        // i will check all 3 areas, if after the applying of the movement, others axis will also collid
        // e.g tti (3,4,5) minimum_col (-3,-3,-3)
        //now after 3 ticks, 4 and 5 still dont Collide
        //but after 5 ticks, 3 is -2 and 4 is -1. and this are still collidung because of minimal collide.
        //so this is our collisison here

        if tti[0].is_sign_negative() && tti[1].is_sign_negative() && tti[2].is_sign_negative() {
            if tti[0] >= tti[1] && tti[0] >= tti[2] {
                return Some(ResolutionTti::Overlapping { since: -tti[0] });
            }
            if tti[1] >= tti[2] && tti[1] >= tti[0] {
                return Some(ResolutionTti::Overlapping { since: -tti[1] });
            }
            if tti[2] >= tti[0] && tti[2] >= tti[1] {
                return Some(ResolutionTti::Overlapping { since: -tti[2] });
            }
            return Some(ResolutionTti::Overlapping { since: -tti[0] }); // UNREACHABLE, except for some infinity stuff
        }

        //doing some sorting here
        #[derive(Debug)]
        struct TtiValueIndex {
            value: f32,
            index: usize,
        }

        impl Ord for TtiValueIndex {
            fn cmp(&self, other: &Self) -> Ordering {
                if self.value.is_infinite() && other.value.is_infinite() {
                    return Ordering::Equal;
                }
                if self.value.is_sign_negative() && other.value.is_sign_negative() {
                    return Ordering::Equal; // we dont want negative
                }
                if self.value.is_sign_negative() {
                    return Ordering::Greater; // be to the end
                }
                if other.value.is_sign_negative() {
                    return Ordering::Less; // be to the end
                }
                if self.value < other.value {
                    return Ordering::Less;
                }
                if self.value > other.value {
                    return Ordering::Greater;
                }
                return Ordering::Equal;
            }
        }

        impl PartialOrd for TtiValueIndex {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
        }

        impl PartialEq for TtiValueIndex {
            fn eq(&self, other: &Self) -> bool { (self.value, &self.index) == (other.value, &other.index) }
        }

        impl Eq for TtiValueIndex {}

        // e.g. (-INF, 4, 2), sort for tti  --> (2, 4)
        let mut to_test = [
            TtiValueIndex {
                value: tti[0],
                index: 0,
            },
            TtiValueIndex {
                value: tti[1],
                index: 1,
            },
            TtiValueIndex {
                value: tti[2],
                index: 2,
            },
        ];
        let mut potentialtouch_index: Option<usize> = None;
        let mut potentialtouch_normal: Option<Vec3<f32>> = None;
        let mut potentialcollide_index: Option<usize> = None;
        let mut potentialcollide_normal: Option<Vec3<f32>> = None;
        to_test.sort();
        //debug!("to_test: {:?}", to_test);
        for i in 0..3 {
            if to_test[i].value >= 0.0 && to_test[i].value.is_finite() {
                //check if others collide after time
                let o1 = (i + 1) % 3;
                let o2 = (i + 2) % 3;
                let o1_i = to_test[o1].index;
                let o2_i = to_test[o2].index;
                // we only shift the value when it actually moves, otherwise min_col is -INF
                let o1_shifted_value = if dire[o1_i] != 0.0 {
                    to_test[o1].value - to_test[i].value
                } else {
                    to_test[o1].value
                };
                let o2_shifted_value = if dire[o2_i] != 0.0 {
                    to_test[o2].value - to_test[i].value
                } else {
                    to_test[o2].value
                };
                //println!("i {}", i);
                //println!("yay: {}, o1 {}, o2 {}", to_test[i].value, o1_shifted_value, o2_shifted_value);
                //println!("max: o1 {}, o2 {}", minimal_collision_tti[o1_i], minimal_collision_tti[o2_i]);
                //println!("dire {:?}", dire);
                //println!("shifted {} {}", o1_shifted_value, o2_shifted_value);
                if (o1_shifted_value < 0.0 || o1_shifted_value == 0.0 && dire[o1_i] != 0.0)
                    && (o1_shifted_value > minimal_collision_tti[o1_i]
                        || (minimal_collision_tti[o1_i].is_infinite()/*&& tti[o1_i] != 0.0*/))
                    && (o2_shifted_value < 0.0 || o2_shifted_value == 0.0 && dire[o2_i] != 0.0)
                    && (o2_shifted_value > minimal_collision_tti[o2_i]
                        || (minimal_collision_tti[o2_i].is_infinite()/*&& tti[o2_i] != 0.0*/))
                {
                    //yep it does, and it's the samllest because to_test was sorted. so output it
                    if dire[to_test[i].index] == 0.0 {
                        // should be return  Some(ResolutionTti::Touching{ normal: normals[to_test[i].index]});
                        if potentialtouch_index.is_none() {
                            potentialtouch_index = Some(i);
                            potentialtouch_normal = Some(normals[to_test[i].index]);
                        }
                    } else {
                        if potentialcollide_index.is_none() {
                            potentialcollide_index = Some(i);
                            potentialcollide_normal = Some(normals[to_test[i].index]);
                        } else if to_test[i].value <= to_test[potentialcollide_index.unwrap()].value {
                            //enge is when 2 or more collect at exact same time
                            if let Some(ref mut nor) = potentialcollide_normal {
                                *nor += normals[to_test[i].index];
                            }
                        }
                    }
                }
            }
        }

        if let Some(i) = potentialcollide_index {
            //info!("returning index: {}, val {}, nor{}", i,  to_test[i].value, potentialcollide_normal.unwrap());
            return Some(ResolutionTti::WillCollide {
                tti: to_test[i].value,
                normal: potentialcollide_normal.unwrap(),
            });
        }

        if let Some(_) = potentialtouch_index {
            return Some(ResolutionTti::Touching {
                normal: potentialtouch_normal.unwrap(),
            });
        }

        return None;
    }

    #[allow(dead_code)]
    pub fn lower(&self) -> Vec3<f32> { self.middle - self.radius }

    #[allow(dead_code)]
    pub fn upper(&self) -> Vec3<f32> { self.middle + self.radius }

    #[allow(dead_code)]
    pub fn middle(&self) -> &Vec3<f32> { &self.middle }
    #[allow(dead_code)]
    pub fn middle_mut(&mut self) -> &mut Vec3<f32> { &mut self.middle }
    #[allow(dead_code)]
    pub fn radius(&self) -> &Vec3<f32> { &self.radius }
    #[allow(dead_code)]
    pub fn radius_mut(&mut self) -> &mut Vec3<f32> { &mut self.radius }
}
