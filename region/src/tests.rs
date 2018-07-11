//Library
use coord::prelude::*;
use rand::prelude::*;

// Parent
use super::collision::{Collidable, Cuboid, Resolution};

fn newmodel(middle: Vec3<f32>, size: Vec3<f32>) -> Collidable {
    let col = Collidable::Cuboid{ cuboid: Cuboid::new(middle, size) };
    return col;
}

#[test]
fn colide_simple() {
    //collide
    let m1 = newmodel(vec3!(0.5, 0.5, 0.5), vec3!(1.0, 1.0, 1.0));
    let m2 = newmodel(vec3!(1.5, 0.5, 0.5), vec3!(1.0, 1.0, 1.0));
    let res = m1.resolve_col(&m2).unwrap();
    assert_eq!(res, Resolution{
        point: vec3!(1.5, 0.5, 0.5),
        correction: vec3!(1.0, 0.0, 0.0),
    });

    let m1 = newmodel(vec3!(0.5, 1.0, 0.5), vec3!(1.0, 1.0, 1.0));
    let m2 = newmodel(vec3!(1.5, 0.5, 0.5), vec3!(1.0, 1.0, 1.0));
    let res = m1.resolve_col(&m2).unwrap();
    assert_eq!(res, Resolution{
        point: vec3!(1.5, 0.5, 0.5),
        correction: vec3!(1.0, 0.0, 0.0),
    });

    // exactly on each other
    let m1 = newmodel(vec3!(0.5, 1.0, 0.5), vec3!(1.0, 1.0, 1.0));
    let m2 = newmodel(vec3!(0.5, 1.0, 0.5), vec3!(1.0, 1.0, 1.0));
    let res = m1.resolve_col(&m2).unwrap();
    assert_eq!(res, Resolution{
        point: vec3!(1.5, 1.0, 0.5),
        correction: vec3!(2.0, 0.0, 0.0),
    });

    let m1 = newmodel(vec3!(0.5, 0.5, 0.5), vec3!(1.0, 1.0, 1.0));
    let m2 = newmodel(vec3!(3.5, 0.5, 0.5), vec3!(1.0, 1.0, 1.0));
    let res = m1.resolve_col(&m2);
    assert!(res.is_none());
}

#[test]
fn touch_simple() {
    //touch
    let m1 = newmodel(vec3!(0.5, 0.5, 0.5), vec3!(0.5, 0.5, 0.5));
    let m2 = newmodel(vec3!(1.5, 0.5, 0.5), vec3!(0.5, 0.5, 0.5));
    let res = m1.resolve_col(&m2).unwrap();
    assert_eq!(res, Resolution{
        point: vec3!(1.0, 0.5, 0.5),
        correction: vec3!(0.0, 0.0, 0.0),
    });
}

#[test]
fn colide_complex() {
    //collide
    let m1 = newmodel(vec3!(0.0, 0.0, 0.0), vec3!(1.0, 1.0, 1.0));
    let m2 = newmodel(vec3!(1.0, 0.5, 0.0), vec3!(1.0, 1.0, 1.0));
    let res = m1.resolve_col(&m2).unwrap();
    assert_eq!(res, Resolution{
        point: vec3!(1.0, 0.5, 0.0),
        correction: vec3!(1.0, 0.0, 0.0),
    });

    let m1 = newmodel(vec3!(0.0, 0.0, 0.0), vec3!(10.0, 10.0, 10.0));
    let m2 = newmodel(vec3!(1.0, 0.5, 0.0), vec3!(1.0, 1.0, 1.0));
    let res = m1.resolve_col(&m2).unwrap();
    assert_eq!(res, Resolution{
        point: vec3!(10.0, 0.5, 0.0),
        correction: vec3!(10.0, 0.0, 0.0),
    });

    let m1 = newmodel(vec3!(0.0, 0.0, 0.0), vec3!(10.0, 10.0, 10.0));
    let m2 = newmodel(vec3!(-1.0, 0.5, 0.0), vec3!(1.0, 1.0, 1.0));
    let res = m1.resolve_col(&m2).unwrap();
    assert_eq!(res, Resolution{
        point: vec3!(-10.0, 0.5, 0.0),
        correction: vec3!(-10.0, 0.0, 0.0),
    });

    //negative
    let m1 = newmodel(vec3!(0.0, 0.0, 0.0), vec3!(10.0, 10.0, 10.0));
    let m2 = newmodel(vec3!(-0.7, -2.0, 0.0), vec3!(1.0, 1.0, 1.0));
    let res = m1.resolve_col(&m2).unwrap();
    assert_eq!(res, Resolution{
        point: vec3!(-0.7, -10.0, 0.0),
        correction: vec3!(0.0, -9.0, 0.0),
    });

    //share a same wall but is inside so overlap
    let m1 = newmodel(vec3!(10.0, 10.0, 10.0), vec3!(10.0, 10.0, 10.0));
    let m2 = newmodel(vec3!(2.0, 6.0, 5.0), vec3!(2.0, 2.0, 2.0));
    let res = m1.resolve_col(&m2).unwrap();
    assert_eq!(res, Resolution{
        point: vec3!(0.0, 6.0, 5.0),
        correction: vec3!(-4.0, 0.0, 0.0),
    });

    // z lies on the surface
    let m1 = newmodel(vec3!(10.0, 10.0, 10.0), vec3!(10.0, 10.0, 10.0));
    let m2 = newmodel(vec3!(8.0, 6.0, 0.0), vec3!(2.0, 2.0, 2.0));
    let res = m1.resolve_col(&m2).unwrap();
    assert_eq!(res, Resolution{
        point: vec3!(8.0, 6.0, 0.0),
        correction: vec3!(0.0, 0.0, -2.0),
    });

    // same but other y
    let m1 = newmodel(vec3!(10.0, 10.0, 10.0), vec3!(10.0, 10.0, 10.0));
    let m2 = newmodel(vec3!(8.0, 7.0, 5.0), vec3!(2.0, 2.0, 2.0));
    let res = m1.resolve_col(&m2).unwrap();
    assert_eq!(res, Resolution{
        point: vec3!(8.0, 7.0, 0.0),
        correction: vec3!(0.0, 0.0, -7.0),
    });

    //outside
    let m1 = newmodel(vec3!(10.0, 10.0, 10.0), vec3!(10.0, 10.0, 10.0));
    let m2 = newmodel(vec3!(22.0, 10.0, 8.0), vec3!(2.0, 2.0, 2.0));
    let res = m1.resolve_col(&m2).unwrap();
    assert_eq!(res, Resolution{
        point: vec3!(20.0, 10.0, 8.0),
        correction: vec3!(0.0, 0.0, 0.0),
    });
}


fn random_vec(scale: f32) -> Vec3<f32> {
    let mut rng = thread_rng();
    let x = ((rng.gen::<f32>())*scale ) as i64 as f32;
    let y = ((rng.gen::<f32>())*scale ) as i64 as f32;
    let z = ((rng.gen::<f32>())*scale ) as i64 as f32;
    Vec3::new(x,y,z)
}

#[test]
fn random_colide_resolution() {
    // choose 1000 random values, if they collide apply resolution, they should now touch
    let mut positive_resolved = 0;

    for _i in 0..1000 {
        let m1 = newmodel(random_vec(10.0)-random_vec(10.0), random_vec(6.0) + vec3!(1.0, 1.0, 1.0));
        let mut m2 = newmodel(random_vec(10.0)-random_vec(10.0), random_vec(6.0) + vec3!(1.0, 1.0, 1.0));
        let res = m1.resolve_col(&m2);
        match res {
            None => (),
            Some(res) => {
                // now apply correction
                if res.isTouch() {
                    continue;
                }
                match &mut m2 {
                    Collidable::Cuboid { ref mut cuboid } => {
                        *cuboid.middle_mut() = *cuboid.middle() + correction;
                    }
                }
                //println!("after {:?}", &m2);
                //println!("ccc {:?}", &correction);
                positive_resolved += 1;
                let res = m1.resolve_col(&m2).unwrap();
                assert_eq!(res, Resolution{ point, .. });
            }
        }
    }
    println!("{} collisions resolved", positive_resolved);
}
