// Standard
use std::{
    collections::HashMap,
    sync::Arc,
    thread,
    time::{self, Duration},
};

//Library
use parking_lot::{Mutex, RwLock};
use rand::prelude::*;
use vek::*;

// Parent
use physics::{
    collision::{Primitive, ResolutionCol, ResolutionTti},
    physics,
};
use terrain::{
    chunk::{Block, Chunk, ChunkContainer, HeterogeneousData},
    BlockLoader, ChunkMgr, ConstructVolume, Container, Entity, ReadWriteVolume, VolCluster, VolGen, VolOffs, VoxRel,
    Voxel,
};
use Uid;

#[test]
fn collide_simple() {
    //collide
    let m1 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, 0.5), Vec3::new(1.0, 1.0, 1.0));
    let m2 = Primitive::new_cuboid(Vec3::new(1.5, 0.5, 0.5), Vec3::new(1.0, 1.0, 1.0));
    let res = m1.resolve_col(&m2).unwrap();
    assert_eq!(
        res,
        ResolutionCol {
            center: Vec3::new(1.0, 0.5, 0.5),
            correction: Vec3::new(1.0, 0.0, 0.0),
        }
    );

    let m1 = Primitive::new_cuboid(Vec3::new(0.5, 1.0, 0.5), Vec3::new(1.0, 1.0, 1.0));
    let m2 = Primitive::new_cuboid(Vec3::new(1.5, 0.5, 0.5), Vec3::new(1.0, 1.0, 1.0));
    let res = m1.resolve_col(&m2).unwrap();
    assert_eq!(
        res,
        ResolutionCol {
            center: Vec3::new(1.0, 0.75, 0.5),
            correction: Vec3::new(1.0, -0.5, 0.0),
        }
    );

    // exactly on each other
    let m1 = Primitive::new_cuboid(Vec3::new(0.5, 1.0, 0.5), Vec3::new(1.0, 1.0, 1.0));
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, 1.0, 0.5), Vec3::new(1.0, 1.0, 1.0));
    let res = m1.resolve_col(&m2).unwrap();
    assert_eq!(
        res,
        ResolutionCol {
            center: Vec3::new(0.5, 1.0, 0.5),
            correction: Vec3::new(0.0, 0.0, 2.0),
        }
    );

    let m1 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, 0.5), Vec3::new(1.0, 1.0, 1.0));
    let m2 = Primitive::new_cuboid(Vec3::new(3.5, 0.5, 0.5), Vec3::new(1.0, 1.0, 1.0));
    let res = m1.resolve_col(&m2);
    assert!(res.is_none());
}

#[test]
fn touch_simple() {
    //touch
    let m1 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, 0.5), Vec3::new(0.5, 0.5, 0.5));
    let m2 = Primitive::new_cuboid(Vec3::new(1.5, 0.5, 0.5), Vec3::new(0.5, 0.5, 0.5));
    let res = m1.resolve_col(&m2).unwrap();
    assert_eq!(
        res,
        ResolutionCol {
            center: Vec3::new(1.0, 0.5, 0.5),
            correction: Vec3::new(0.0, 0.0, 0.0),
        }
    );
}

#[test]
fn collide_complex() {
    //collide
    let m1 = Primitive::new_cuboid(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0));
    let m2 = Primitive::new_cuboid(Vec3::new(1.0, 0.5, 0.0), Vec3::new(1.0, 1.0, 1.0));
    let res = m1.resolve_col(&m2).unwrap();
    assert_eq!(
        res,
        ResolutionCol {
            center: Vec3::new(0.5, 0.25, 0.0),
            correction: Vec3::new(1.0, 0.5, 0.0),
        }
    );

    let m1 = Primitive::new_cuboid(Vec3::new(0.0, 0.0, 0.0), Vec3::new(10.0, 10.0, 10.0));
    let m2 = Primitive::new_cuboid(Vec3::new(1.0, 0.5, 0.0), Vec3::new(1.0, 1.0, 1.0));
    let res = m1.resolve_col(&m2).unwrap();
    assert_eq!(
        res,
        ResolutionCol {
            center: Vec3::new(0.5, 0.25, 0.0),
            correction: Vec3::new(10.0, 5.0, 0.0),
        }
    );

    let m1 = Primitive::new_cuboid(Vec3::new(0.0, 0.0, 0.0), Vec3::new(10.0, 10.0, 10.0));
    let m2 = Primitive::new_cuboid(Vec3::new(-1.0, 0.5, 0.0), Vec3::new(1.0, 1.0, 1.0));
    let res = m1.resolve_col(&m2).unwrap();
    assert_eq!(
        res,
        ResolutionCol {
            center: Vec3::new(-0.5, 0.25, 0.0),
            correction: Vec3::new(-10.0, 5.0, 0.0),
        }
    );

    //negative
    let m1 = Primitive::new_cuboid(Vec3::new(0.0, 0.0, 0.0), Vec3::new(10.0, 10.0, 10.0));
    let m2 = Primitive::new_cuboid(Vec3::new(-0.7, -2.0, 0.0), Vec3::new(1.0, 1.0, 1.0));
    let res = m1.resolve_col(&m2).unwrap();
    assert_eq!(res.center, Vec3::new(-0.35, -1.0, 0.0));
    let rounded = res.correction.map(|e| (e * 100.0).round() / 100.0);
    assert_eq!(rounded, Vec3::new(-3.15, -9.0, 0.0));

    //share a same wall but is inside so overlap
    let m1 = Primitive::new_cuboid(Vec3::new(10.0, 10.0, 10.0), Vec3::new(10.0, 10.0, 10.0));
    let m2 = Primitive::new_cuboid(Vec3::new(2.0, 6.0, 5.0), Vec3::new(2.0, 2.0, 2.0));
    let res = m1.resolve_col(&m2).unwrap();
    assert_eq!(
        res,
        ResolutionCol {
            center: Vec3::new(6.0, 8.0, 7.5),
            correction: Vec3::new(-4.0, -2.0, -2.5),
        }
    );

    // z lies on the surface
    let m1 = Primitive::new_cuboid(Vec3::new(10.0, 10.0, 10.0), Vec3::new(10.0, 10.0, 10.0));
    let m2 = Primitive::new_cuboid(Vec3::new(8.0, 6.0, 0.0), Vec3::new(2.0, 2.0, 2.0));
    let res = m1.resolve_col(&m2).unwrap();
    assert_eq!(
        res,
        ResolutionCol {
            center: Vec3::new(9.0, 8.0, 5.0),
            correction: Vec3::new(-0.4, -0.8, -2.0),
        }
    );

    // same but other y
    let m1 = Primitive::new_cuboid(Vec3::new(10.0, 10.0, 10.0), Vec3::new(10.0, 10.0, 10.0));
    let m2 = Primitive::new_cuboid(Vec3::new(8.0, 7.0, 5.0), Vec3::new(2.0, 2.0, 2.0));
    let res = m1.resolve_col(&m2).unwrap();
    assert_eq!(
        res,
        ResolutionCol {
            center: Vec3::new(9.0, 8.5, 7.5),
            correction: Vec3::new(-2.8, -4.2, -7.0),
        }
    );

    //outside
    let m1 = Primitive::new_cuboid(Vec3::new(10.0, 10.0, 10.0), Vec3::new(10.0, 10.0, 10.0));
    let m2 = Primitive::new_cuboid(Vec3::new(22.0, 10.0, 8.0), Vec3::new(2.0, 2.0, 2.0));
    let res = m1.resolve_col(&m2).unwrap();
    assert_eq!(
        res,
        ResolutionCol {
            center: Vec3::new(16.0, 10.0, 9.0),
            correction: Vec3::new(0.0, 0.0, 0.0),
        }
    );
}

#[test]
fn touch_wall() {
    // Simulate a wall touch
    let w1 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, 0.5), Vec3::new(0.5, 0.5, 0.5));
    let w2 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, 1.5), Vec3::new(0.5, 0.5, 0.5));
    let w3 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, 2.5), Vec3::new(0.5, 0.5, 0.5));
    let w4 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, 3.5), Vec3::new(0.5, 0.5, 0.5));
    let m1 = Primitive::new_cuboid(Vec3::new(1.45, 0.51234, 1.2), Vec3::new(0.45, 0.45, 0.9));
    let res = w1.resolve_col(&m1).unwrap();
    assert_eq!(
        res,
        ResolutionCol {
            center: Vec3::new(0.975, 0.50617003, 0.85),
            correction: Vec3::new(0.0, 0.0, 0.0),
        }
    );

    //assert_eq!(res.center, Vec3::new(1.0, 0.51234, 1.2));
    assert!(res.is_touch());
    let res = w2.resolve_col(&m1).unwrap();
    assert_eq!(
        res,
        ResolutionCol {
            center: Vec3::new(0.975, 0.50617003, 1.35),
            correction: Vec3::new(0.0, 0.0, 0.0),
        }
    );
    //assert_eq!(res.center, Vec3::new(1.0, 0.51234, 1.2));
    assert!(res.is_touch());
    let res = w3.resolve_col(&m1).unwrap();
    //assert_eq!(res.center, Vec3::new(1.0, 0.51234, 1.2));
    assert_eq!(
        res,
        ResolutionCol {
            center: Vec3::new(0.975, 0.50617003, 1.85),
            correction: Vec3::new(0.0, 0.0, 0.0),
        }
    );
    assert!(res.is_touch());
    let res = w4.resolve_col(&m1);
    assert_eq!(res, None);
}

fn random_vec(scale: f32) -> Vec3<f32> {
    let mut rng = thread_rng();
    let x = ((rng.gen::<f32>()) * scale) as i64 as f32;
    let y = ((rng.gen::<f32>()) * scale) as i64 as f32;
    let z = ((rng.gen::<f32>()) * scale) as i64 as f32;
    Vec3::new(x, y, z)
}

#[test]
fn random_collide_resolution() {
    // choose 1000 random values, if they collide apply resolution, they should now touch
    let mut positive_resolved = 0;

    for _i in 0..1000 {
        let m1 = Primitive::new_cuboid(
            random_vec(10.0) - random_vec(10.0),
            random_vec(6.0) + Vec3::new(1.0, 1.0, 1.0),
        );
        let mut m2 = Primitive::new_cuboid(
            random_vec(10.0) - random_vec(10.0),
            random_vec(6.0) + Vec3::new(1.0, 1.0, 1.0),
        );
        let res = m1.resolve_col(&m2);
        match res {
            None => (),
            Some(res) => {
                // now apply correction
                if res.is_touch() {
                    continue;
                }
                m2.move_by(&res.correction);
                //println!("after {:?}", &m2);
                //println!("ccc {:?}", &correction);
                positive_resolved += 1;
                let res = m1.resolve_col(&m2).unwrap();
                assert!(res.is_touch());
            },
        }
    }
    println!("{} collisions resolved", positive_resolved);
}

/*
//#[test]
fn tti_simple() {
    //touch
    let m1 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, 0.5), Vec3::new(0.5, 0.5, 0.5));
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, 10.5), Vec3::new(0.5, 0.5, 0.5));
    let res = m1.time_to_impact(&m2, &Vec3::new(0.0, 0.0, -1.0)).unwrap();
    assert_eq!(res.0, 9.0);
    let res = m1.time_to_impact(&m2, &Vec3::new(0.0, 0.0, 1.0));
    assert!(res.is_none());
    let res = m1.time_to_impact(&m2, &Vec3::new(0.0, 0.0, -0.1)).unwrap();
    assert_eq!(res.0, 90.0);

    let m2 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, 10.5), Vec3::new(0.5, 0.5, 1.5));
    let res = m1.time_to_impact(&m2, &Vec3::new(0.0, 0.0, -1.0)).unwrap();
    assert_eq!(res.0, 8.0);
    let m2 = Primitive::new_cuboid(Vec3::new(0.75, -0.25, 10.5), Vec3::new(0.5, 0.5, 1.5));
    let res = m1.time_to_impact(&m2, &Vec3::new(0.0, 0.0, -1.0)).unwrap();
    assert_eq!(res.0, 8.0);
    let m2 = Primitive::new_cuboid(Vec3::new(0.75, -0.5, 10.5), Vec3::new(0.5, 0.5, 1.5));
    let res = m1.time_to_impact(&m2, &Vec3::new(0.0, 0.0, -1.0)).unwrap();
    assert_eq!(res.0, 8.0);
    let m2 = Primitive::new_cuboid(Vec3::new(0.75, -0.75, 10.5), Vec3::new(0.5, 0.5, 1.5));
    let res = m1.time_to_impact(&m2, &Vec3::new(0.0, 0.0, -1.0));
    assert!(res.is_none());

    let m2 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, -9.5), Vec3::new(0.5, 0.5, 0.5));
    let res = m1.time_to_impact(&m2, &Vec3::new(0.0, 0.0, 1.0)).unwrap();
    assert_eq!(res.0, 9.0);
    let res = m1.time_to_impact(&m2, &Vec3::new(0.0, 0.0, -1.0));
    assert!(res.is_none());
    let res = m1.time_to_impact(&m2, &Vec3::new(0.0, 0.0, 0.1)).unwrap();
    assert_eq!(res.0, 90.0);
}*/

macro_rules! checkWillCollide {
    ($x:expr, $tti2:expr, $normal2:expr) => {
        let res = $x;
        //println!("EXPT: {:?} {:?}", $tti2, $normal2);
        //println!("{:?}", res);
        assert!(res.is_some());
        let res = res.expect("Does not collide ever");
        if let ResolutionTti::WillCollide { tti, normal } = res {
            let cmp = ((tti * 1000.0) as f32).round() / 1000.0;
            assert_eq!(cmp, $tti2);
            assert_eq!(normal, $normal2);
        } else {
            panic!("wrong collision type: {:?}", res);
        }
    };
}

macro_rules! checkTouching {
    ($x:expr, $normal2:expr) => {
        let res = $x;
        //println!("EXPT: {:?}", $normal2);
        //println!("{:?}", res);
        assert!(res.is_some());
        let res = res.expect("Does not collide ever");
        if let ResolutionTti::Touching { normal } = res {
            assert_eq!(normal, $normal2);
        } else {
            panic!("wrong collision type: {:?}", res);
        }
    };
}

macro_rules! checkOverlapping {
    ($x:expr, $since2:expr) => {
        let res = $x;
        //println!("EXPT: {:?}", $since2);
        //println!("{:?}", res);
        assert!(res.is_some());
        let res = res.expect("Does not collide ever");
        if let ResolutionTti::Overlapping { since } = res {
            let cmp = ((since * 1000.0) as f32).round() / 1000.0;
            assert_eq!(cmp, $since2);
        } else {
            panic!("wrong collision type: {:?}", res);
        }
    };
}

macro_rules! checkNone {
    ($x:expr) => {
        let res = $x;
        //println!("{:?}", res);
        assert!(res.is_none());
    };
}

#[test]
fn tti_horizontal_positions_const_vel() {
    let vel = Vec3::new(0.0, 0.0, -1.0);
    let normal = Vec3::new(0.0, 0.0, 1.0);
    let m1 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, 0.5), Vec3::new(0.5, 0.5, 0.5));
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, 1000.5), Vec3::new(0.5, 0.5, 0.5));
    checkWillCollide!(m1.time_to_impact(&m2, &vel), 999.0, normal);
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, 3.5), Vec3::new(0.5, 0.5, 0.5));
    checkWillCollide!(m1.time_to_impact(&m2, &vel), 2.0, normal);
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, 2.5), Vec3::new(0.5, 0.5, 0.5));
    checkWillCollide!(m1.time_to_impact(&m2, &vel), 1.0, normal);
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, 2.0), Vec3::new(0.5, 0.5, 0.5));
    checkWillCollide!(m1.time_to_impact(&m2, &vel), 0.5, normal);
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, 1.51), Vec3::new(0.5, 0.5, 0.5));
    checkWillCollide!(m1.time_to_impact(&m2, &vel), 0.01, normal);
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, 1.5), Vec3::new(0.5, 0.5, 0.5));
    checkWillCollide!(m1.time_to_impact(&m2, &vel), 0.0, normal);
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, 1.49), Vec3::new(0.5, 0.5, 0.5));
    checkOverlapping!(m1.time_to_impact(&m2, &vel), 0.01);
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, 1.0), Vec3::new(0.5, 0.5, 0.5));
    checkOverlapping!(m1.time_to_impact(&m2, &vel), 0.5);
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, 0.5), Vec3::new(0.5, 0.5, 0.5));
    checkOverlapping!(m1.time_to_impact(&m2, &vel), 1.0);
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, 0.0), Vec3::new(0.5, 0.5, 0.5));
    checkOverlapping!(m1.time_to_impact(&m2, &vel), 1.5);
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, -0.4), Vec3::new(0.5, 0.5, 0.5));
    checkOverlapping!(m1.time_to_impact(&m2, &vel), 1.9);
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, -0.5), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel)); // touching on the other side is no longer considered as touching. time is up!
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, -0.6), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, -1.5), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, -2.5), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, -20.5), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, -112.5), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
}

#[test]
fn tti_horizontal_positions_const_vel_negative() {
    let vel = Vec3::new(0.0, 0.0, 1.0);
    let normal = Vec3::new(0.0, 0.0, -1.0);
    let m1 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, 0.5), Vec3::new(0.5, 0.5, 0.5));
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, -999.5), Vec3::new(0.5, 0.5, 0.5));
    checkWillCollide!(m1.time_to_impact(&m2, &vel), 999.0, normal);
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, -2.5), Vec3::new(0.5, 0.5, 0.5));
    checkWillCollide!(m1.time_to_impact(&m2, &vel), 2.0, normal);
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, -1.5), Vec3::new(0.5, 0.5, 0.5));
    checkWillCollide!(m1.time_to_impact(&m2, &vel), 1.0, normal);
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, -1.0), Vec3::new(0.5, 0.5, 0.5));
    checkWillCollide!(m1.time_to_impact(&m2, &vel), 0.5, normal);
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, -0.51), Vec3::new(0.5, 0.5, 0.5));
    checkWillCollide!(m1.time_to_impact(&m2, &vel), 0.01, normal);
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, -0.5), Vec3::new(0.5, 0.5, 0.5));
    checkWillCollide!(m1.time_to_impact(&m2, &vel), 0.0, normal);
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, -0.49), Vec3::new(0.5, 0.5, 0.5));
    checkOverlapping!(m1.time_to_impact(&m2, &vel), 0.01);
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, 0.0), Vec3::new(0.5, 0.5, 0.5));
    checkOverlapping!(m1.time_to_impact(&m2, &vel), 0.5);
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, 0.5), Vec3::new(0.5, 0.5, 0.5));
    checkOverlapping!(m1.time_to_impact(&m2, &vel), 1.0);
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, 1.0), Vec3::new(0.5, 0.5, 0.5));
    checkOverlapping!(m1.time_to_impact(&m2, &vel), 1.5);
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, 1.4), Vec3::new(0.5, 0.5, 0.5));
    checkOverlapping!(m1.time_to_impact(&m2, &vel), 1.9);
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, 1.5), Vec3::new(0.5, 0.5, 0.5));
    let res = m1.time_to_impact(&m2, &vel);
    assert!(res.is_none()); // touching on the other side is no longer considered as touching. time is up!
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, 1.6), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, 2.5), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, 3.5), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, 21.5), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, 113.5), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
}

#[test]
fn tti_horizontal_positions_const_vel_beneath() {
    let vel = Vec3::new(0.0, 0.0, -1.0);
    let m1 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, 0.5), Vec3::new(0.5, 0.5, 0.5));
    let m2 = Primitive::new_cuboid(Vec3::new(5.5, 0.5, 1000.5), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
    let m2 = Primitive::new_cuboid(Vec3::new(5.5, 0.5, 3.5), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
    let m2 = Primitive::new_cuboid(Vec3::new(5.5, 0.5, 2.5), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
    let m2 = Primitive::new_cuboid(Vec3::new(5.5, 0.5, 2.0), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
    let m2 = Primitive::new_cuboid(Vec3::new(5.5, 0.5, 1.51), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
    let m2 = Primitive::new_cuboid(Vec3::new(5.5, 0.5, 1.5), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
    let m2 = Primitive::new_cuboid(Vec3::new(5.5, 0.5, 1.49), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
    let m2 = Primitive::new_cuboid(Vec3::new(5.5, 0.5, 1.0), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
    let m2 = Primitive::new_cuboid(Vec3::new(5.5, 0.5, 0.5), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
    let m2 = Primitive::new_cuboid(Vec3::new(5.5, 0.5, 0.0), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
    let m2 = Primitive::new_cuboid(Vec3::new(5.5, 0.5, -0.4), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
    let m2 = Primitive::new_cuboid(Vec3::new(5.5, 0.5, -0.5), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
    let m2 = Primitive::new_cuboid(Vec3::new(5.5, 0.5, -0.6), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
    let m2 = Primitive::new_cuboid(Vec3::new(5.5, 0.5, -1.5), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
    let m2 = Primitive::new_cuboid(Vec3::new(5.5, 0.5, -2.5), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
    let m2 = Primitive::new_cuboid(Vec3::new(5.5, 0.5, -20.5), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
    let m2 = Primitive::new_cuboid(Vec3::new(5.5, 0.5, -112.5), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
}

#[test]
fn tti_horizontal_positions_const_vel_beneath_touching() {
    let vel = Vec3::new(0.0, 0.0, -1.0);
    let normal = Vec3::new(1.0, 0.0, 0.0);
    let m1 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, 0.5), Vec3::new(0.5, 0.5, 0.5));
    let m2 = Primitive::new_cuboid(Vec3::new(1.5, 0.5, 1000.5), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
    let m2 = Primitive::new_cuboid(Vec3::new(1.5, 0.5, 3.5), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
    let m2 = Primitive::new_cuboid(Vec3::new(1.5, 0.5, 2.5), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
    let m2 = Primitive::new_cuboid(Vec3::new(1.5, 0.5, 2.0), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
    let m2 = Primitive::new_cuboid(Vec3::new(1.5, 0.5, 1.51), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
    let m2 = Primitive::new_cuboid(Vec3::new(1.5, 0.5, 1.5), Vec3::new(0.5, 0.5, 0.5));
    checkTouching!(m1.time_to_impact(&m2, &vel), normal); // should it be couhing + edge here?
    let m2 = Primitive::new_cuboid(Vec3::new(1.5, 0.5, 1.49), Vec3::new(0.5, 0.5, 0.5));
    checkTouching!(m1.time_to_impact(&m2, &vel), normal);
    let m2 = Primitive::new_cuboid(Vec3::new(1.5, 0.5, 1.0), Vec3::new(0.5, 0.5, 0.5));
    checkTouching!(m1.time_to_impact(&m2, &vel), normal);
    let m2 = Primitive::new_cuboid(Vec3::new(1.5, 0.5, 0.5), Vec3::new(0.5, 0.5, 0.5));
    checkTouching!(m1.time_to_impact(&m2, &vel), normal);
    let m2 = Primitive::new_cuboid(Vec3::new(1.5, 0.5, 0.0), Vec3::new(0.5, 0.5, 0.5));
    checkTouching!(m1.time_to_impact(&m2, &vel), normal);
    let m2 = Primitive::new_cuboid(Vec3::new(1.5, 0.5, -0.4), Vec3::new(0.5, 0.5, 0.5));
    checkTouching!(m1.time_to_impact(&m2, &vel), normal);
    let m2 = Primitive::new_cuboid(Vec3::new(1.5, 0.5, -0.5), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel)); // shouldnt this touch?
    let m2 = Primitive::new_cuboid(Vec3::new(1.5, 0.5, -0.6), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
    let m2 = Primitive::new_cuboid(Vec3::new(1.5, 0.5, -1.5), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
    let m2 = Primitive::new_cuboid(Vec3::new(1.5, 0.5, -2.5), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
    let m2 = Primitive::new_cuboid(Vec3::new(1.5, 0.5, -20.5), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
    let m2 = Primitive::new_cuboid(Vec3::new(1.5, 0.5, -112.5), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
}

#[test]
fn tti_horizontal_positions_const_vel_beneath_touching_negative() {
    let vel = Vec3::new(0.0, 0.0, -1.0);
    let normal = Vec3::new(-1.0, 0.0, 0.0);
    let m1 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, 0.5), Vec3::new(0.5, 0.5, 0.5));
    let m2 = Primitive::new_cuboid(Vec3::new(-0.5, 0.5, 1000.5), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
    let m2 = Primitive::new_cuboid(Vec3::new(-0.5, 0.5, 3.5), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
    let m2 = Primitive::new_cuboid(Vec3::new(-0.5, 0.5, 2.5), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
    let m2 = Primitive::new_cuboid(Vec3::new(-0.5, 0.5, 2.0), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
    let m2 = Primitive::new_cuboid(Vec3::new(-0.5, 0.5, 1.51), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
    let m2 = Primitive::new_cuboid(Vec3::new(-0.5, 0.5, 1.5), Vec3::new(0.5, 0.5, 0.5));
    checkTouching!(m1.time_to_impact(&m2, &vel), normal); // should it be couhing + edge here?
    let m2 = Primitive::new_cuboid(Vec3::new(-0.5, 0.5, 1.49), Vec3::new(0.5, 0.5, 0.5));
    checkTouching!(m1.time_to_impact(&m2, &vel), normal);
    let m2 = Primitive::new_cuboid(Vec3::new(-0.5, 0.5, 1.0), Vec3::new(0.5, 0.5, 0.5));
    checkTouching!(m1.time_to_impact(&m2, &vel), normal);
    let m2 = Primitive::new_cuboid(Vec3::new(-0.5, 0.5, 0.5), Vec3::new(0.5, 0.5, 0.5));
    checkTouching!(m1.time_to_impact(&m2, &vel), normal);
    let m2 = Primitive::new_cuboid(Vec3::new(-0.5, 0.5, 0.0), Vec3::new(0.5, 0.5, 0.5));
    checkTouching!(m1.time_to_impact(&m2, &vel), normal);
    let m2 = Primitive::new_cuboid(Vec3::new(-0.5, 0.5, -0.4), Vec3::new(0.5, 0.5, 0.5));
    checkTouching!(m1.time_to_impact(&m2, &vel), normal);
    let m2 = Primitive::new_cuboid(Vec3::new(-0.5, 0.5, -0.5), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel)); // shouldnt this touch?
    let m2 = Primitive::new_cuboid(Vec3::new(-0.5, 0.5, -0.6), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
    let m2 = Primitive::new_cuboid(Vec3::new(-0.5, 0.5, -1.5), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
    let m2 = Primitive::new_cuboid(Vec3::new(-0.5, 0.5, -2.5), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
    let m2 = Primitive::new_cuboid(Vec3::new(-0.5, 0.5, -20.5), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
    let m2 = Primitive::new_cuboid(Vec3::new(-0.5, 0.5, -112.5), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
}

#[test]
fn tti_horizontal_positions_const_vel2() {
    let vel = Vec3::new(0.0, 0.0, -1.0);
    let normal = Vec3::new(0.0, 0.0, 1.0);
    let m1 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, 0.5), Vec3::new(0.5, 0.5, 0.5));
    let m2 = Primitive::new_cuboid(Vec3::new(1.0, 0.5, 1000.5), Vec3::new(0.5, 0.5, 0.5));
    checkWillCollide!(m1.time_to_impact(&m2, &vel), 999.0, normal);
    let m2 = Primitive::new_cuboid(Vec3::new(1.0, 0.5, 3.5), Vec3::new(0.5, 0.5, 0.5));
    checkWillCollide!(m1.time_to_impact(&m2, &vel), 2.0, normal);
    let m2 = Primitive::new_cuboid(Vec3::new(1.0, 0.5, 2.5), Vec3::new(0.5, 0.5, 0.5));
    checkWillCollide!(m1.time_to_impact(&m2, &vel), 1.0, normal);
    let m2 = Primitive::new_cuboid(Vec3::new(1.0, 0.5, 2.0), Vec3::new(0.5, 0.5, 0.5));
    checkWillCollide!(m1.time_to_impact(&m2, &vel), 0.5, normal);
    let m2 = Primitive::new_cuboid(Vec3::new(1.0, 0.5, 1.51), Vec3::new(0.5, 0.5, 0.5));
    checkWillCollide!(m1.time_to_impact(&m2, &vel), 0.01, normal);
    let m2 = Primitive::new_cuboid(Vec3::new(1.0, 0.5, 1.5), Vec3::new(0.5, 0.5, 0.5));
    checkWillCollide!(m1.time_to_impact(&m2, &vel), 0.0, normal);
    let m2 = Primitive::new_cuboid(Vec3::new(1.0, 0.5, 1.49), Vec3::new(0.5, 0.5, 0.5));
    checkOverlapping!(m1.time_to_impact(&m2, &vel), 0.01);
    let m2 = Primitive::new_cuboid(Vec3::new(1.0, 0.5, 1.0), Vec3::new(0.5, 0.5, 0.5));
    checkOverlapping!(m1.time_to_impact(&m2, &vel), 0.5);
    let m2 = Primitive::new_cuboid(Vec3::new(1.0, 0.5, 0.5), Vec3::new(0.5, 0.5, 0.5));
    checkOverlapping!(m1.time_to_impact(&m2, &vel), 1.0);
    let m2 = Primitive::new_cuboid(Vec3::new(1.0, 0.5, 0.0), Vec3::new(0.5, 0.5, 0.5));
    checkOverlapping!(m1.time_to_impact(&m2, &vel), 1.5);
    let m2 = Primitive::new_cuboid(Vec3::new(1.0, 0.5, -0.4), Vec3::new(0.5, 0.5, 0.5));
    checkOverlapping!(m1.time_to_impact(&m2, &vel), 1.9);
    let m2 = Primitive::new_cuboid(Vec3::new(1.0, 0.5, -0.5), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel)); // touching on the other side is no longer considered as touching. time is up!
    let m2 = Primitive::new_cuboid(Vec3::new(1.0, 0.5, -0.6), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
    let m2 = Primitive::new_cuboid(Vec3::new(1.0, 0.5, -1.5), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
    let m2 = Primitive::new_cuboid(Vec3::new(1.0, 0.5, -2.5), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
    let m2 = Primitive::new_cuboid(Vec3::new(1.0, 0.5, -20.5), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
    let m2 = Primitive::new_cuboid(Vec3::new(1.0, 0.5, -112.5), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
}

#[test]
fn tti_diagonal_positions_const_vel() {
    let vel = Vec3::new(0.0, -0.5, -1.0);
    let top = Vec3::new(0.0, 0.0, 1.0);
    let side = Vec3::new(0.0, 1.0, 0.0);
    // z < 3.5 no longer hit side
    // z > 9.5 no longer hit top

    let m1 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, 0.5), Vec3::new(0.5, 0.5, 0.5));
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, 3.5, 2.5), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, 3.5, 3.5), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel)); //touch the edge
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, 3.5, 4.5), Vec3::new(0.5, 0.5, 0.5));
    checkWillCollide!(m1.time_to_impact(&m2, &vel), 4.0, side);
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, 3.5, 5.0), Vec3::new(0.5, 0.5, 0.5));
    checkWillCollide!(m1.time_to_impact(&m2, &vel), 4.0, side);
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, 3.5, 5.5), Vec3::new(0.5, 0.5, 0.5));
    checkWillCollide!(m1.time_to_impact(&m2, &vel), 4.0, side + top); // edge
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, 3.5, 6.0), Vec3::new(0.5, 0.5, 0.5));
    checkWillCollide!(m1.time_to_impact(&m2, &vel), 4.5, top);
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, 3.5, 7.0), Vec3::new(0.5, 0.5, 0.5));
    checkWillCollide!(m1.time_to_impact(&m2, &vel), 5.5, top);
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, 3.5, 8.0), Vec3::new(0.5, 0.5, 0.5));
    checkWillCollide!(m1.time_to_impact(&m2, &vel), 6.5, top);
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, 3.5, 9.0), Vec3::new(0.5, 0.5, 0.5));
    checkWillCollide!(m1.time_to_impact(&m2, &vel), 7.5, top);
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, 3.5, 9.5), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel)); //touch the edge
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, 3.5, 10.0), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
}

#[test]
fn tti_diagonal_positions_const_vel_negative() {
    let vel = Vec3::new(0.0, 0.5, -1.0);
    let top = Vec3::new(0.0, 0.0, 1.0);
    let side = Vec3::new(0.0, -1.0, 0.0);
    // z < 3.5 no longer hit side
    // z > 9.5 no longer hit top
    let m1 = Primitive::new_cuboid(Vec3::new(0.5, 0.5, 0.5), Vec3::new(0.5, 0.5, 0.5));
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, -2.5, 2.5), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, -2.5, 3.5), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel)); //touch the edge
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, -2.5, 4.5), Vec3::new(0.5, 0.5, 0.5));
    checkWillCollide!(m1.time_to_impact(&m2, &vel), 4.0, side);
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, -2.5, 5.0), Vec3::new(0.5, 0.5, 0.5));
    checkWillCollide!(m1.time_to_impact(&m2, &vel), 4.0, side);
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, -2.5, 5.5), Vec3::new(0.5, 0.5, 0.5));
    checkWillCollide!(m1.time_to_impact(&m2, &vel), 4.0, side + top); // edge
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, -2.5, 6.0), Vec3::new(0.5, 0.5, 0.5));
    checkWillCollide!(m1.time_to_impact(&m2, &vel), 4.5, top);
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, -2.5, 7.0), Vec3::new(0.5, 0.5, 0.5));
    checkWillCollide!(m1.time_to_impact(&m2, &vel), 5.5, top);
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, -2.5, 8.0), Vec3::new(0.5, 0.5, 0.5));
    checkWillCollide!(m1.time_to_impact(&m2, &vel), 6.5, top);
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, -2.5, 9.0), Vec3::new(0.5, 0.5, 0.5));
    checkWillCollide!(m1.time_to_impact(&m2, &vel), 7.5, top);
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, -2.5, 9.5), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel)); //touch the edge
    let m2 = Primitive::new_cuboid(Vec3::new(0.5, -2.5, 10.0), Vec3::new(0.5, 0.5, 0.5));
    checkNone!(m1.time_to_impact(&m2, &vel));
}

#[test]
fn tti_diagonal_in_to_dirs() {
    let vel = Vec3::new(-0.46, 0.0, 0.0);
    let normal = Vec3::new(0.0, 1.0, 0.0);
    let m1 = Primitive::new_cuboid(Vec3::new(-105.5, -68.5, 118.5), Vec3::new(0.5, 0.5, 0.5));
    let m2 = Primitive::new_cuboid(Vec3::new(-104.55, -67.55, 118.9), Vec3::new(0.45, 0.45, 0.9));
    checkTouching!(m1.time_to_impact(&m2, &vel), normal);
    let vel = Vec3::new(0.0, -0.37, 0.0);
    let normal = Vec3::new(1.0, 0.0, 0.0);
    checkTouching!(m1.time_to_impact(&m2, &vel), normal);
}

#[test]
fn tti_diagonal_in_to_dirs_negative() {
    let vel = Vec3::new(0.46, 0.0, 0.0);
    let normal = Vec3::new(0.0, -1.0, 0.0);
    let m1 = Primitive::new_cuboid(Vec3::new(105.5, 68.5, -118.5), Vec3::new(0.5, 0.5, 0.5));
    let m2 = Primitive::new_cuboid(Vec3::new(104.55, 67.55, -118.9), Vec3::new(0.45, 0.45, 0.9));
    checkTouching!(m1.time_to_impact(&m2, &vel), normal);
    let vel = Vec3::new(0.0, 0.37, 0.0);
    let normal = Vec3::new(-1.0, 0.0, 0.0);
    checkTouching!(m1.time_to_impact(&m2, &vel), normal);
}

// Constants
pub const CHUNK_SIZE: Vec3<VoxRel> = Vec3 { x: 64, y: 64, z: 64 }; // TODO: Unify this using the chunk interface
pub const CHUNK_MID: Vec3<f32> = Vec3 {
    x: CHUNK_SIZE.x as f32 / 2.0,
    y: CHUNK_SIZE.y as f32 / 2.0,
    z: CHUNK_SIZE.z as f32 / 2.0,
};

fn gen_chunk_flat(_pos: Vec3<VolOffs>, con: Arc<Mutex<Option<ChunkContainer<i64>>>>) {
    let mut c = HeterogeneousData::empty(CHUNK_SIZE);
    for x in 0..CHUNK_SIZE.x {
        for y in 0..CHUNK_SIZE.y {
            c.replace_at_unchecked(Vec3::new(x, y, 2), Block::STONE);
        }
    }
    *con.lock() = Some(ChunkContainer::<i64>::new(Chunk::Hetero(c)));
}

fn gen_chunk_flat_border(_pos: Vec3<VolOffs>, con: Arc<Mutex<Option<ChunkContainer<i64>>>>) {
    let mut c = HeterogeneousData::empty(CHUNK_SIZE);
    for x in 0..CHUNK_SIZE.x {
        for y in 0..CHUNK_SIZE.y {
            c.replace_at_unchecked(Vec3::new(x, y, 2), Block::STONE);
        }
    }
    for i in 0..CHUNK_SIZE.x {
        c.replace_at_unchecked(Vec3::new(i, 0, 3), Block::STONE);
        c.replace_at_unchecked(Vec3::new(i, CHUNK_SIZE.x - 1, 3), Block::STONE);
        c.replace_at_unchecked(Vec3::new(0, i, 3), Block::STONE);
        c.replace_at_unchecked(Vec3::new(CHUNK_SIZE.x - 1, i, 3), Block::STONE);

        c.replace_at_unchecked(Vec3::new(i, 0, 4), Block::STONE);
        c.replace_at_unchecked(Vec3::new(i, CHUNK_SIZE.x - 1, 4), Block::STONE);
        c.replace_at_unchecked(Vec3::new(0, i, 4), Block::STONE);
        c.replace_at_unchecked(Vec3::new(CHUNK_SIZE.x - 1, i, 4), Block::STONE);
    }
    *con.lock() = Some(ChunkContainer::<i64>::new(Chunk::Hetero(c)));
}

fn gen_payload(_pos: Vec3<VolOffs>, con: Arc<Mutex<Option<ChunkContainer<i64>>>>) {
    let conlock = con.lock();
    if let Some(ref con) = *conlock {
        *con.payload_mut() = Some(42);
    }
}

fn drop_chunk(_pos: Vec3<VolOffs>, _con: Arc<ChunkContainer<i64>>) {}

fn drop_payload(_pos: Vec3<VolOffs>, _con: Arc<ChunkContainer<i64>>) {}

#[test]
fn physics_fall() {
    let vol_mgr = ChunkMgr::new(
        CHUNK_SIZE,
        VolGen::new(gen_chunk_flat, gen_payload, drop_chunk, drop_payload),
    );
    vol_mgr.block_loader_mut().push(Arc::new(RwLock::new(BlockLoader {
        pos: Vec3::new(0, 0, 0),
        size: CHUNK_SIZE.map(|e| e as i64 * 10),
    })));
    vol_mgr.gen(Vec3::new(0, 0, 0));
    vol_mgr.gen(Vec3::new(0, 0, -1));
    thread::sleep(time::Duration::from_millis(200)); // because this spawns a thread :/
                                                     //touch
    vol_mgr.maintain();
    let mut ent: HashMap<Uid, Arc<RwLock<Entity<()>>>> = HashMap::new();
    ent.insert(
        1,
        Arc::new(RwLock::new(Entity::new(
            Vec3::new(CHUNK_MID.x, CHUNK_MID.y, 10.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec2::new(0.0, 0.0),
        ))),
    );
    for _ in 0..40 {
        physics::tick(ent.values(), &vol_mgr, Duration::from_millis(100))
    }
    let p = ent.get(&1);
    let d = *p.unwrap().read().pos() - Vec3::new(CHUNK_MID.x, CHUNK_MID.y, 3.0);
    println!("{}, physics_fall {}", d.magnitude(), *p.unwrap().read().pos());
    assert!(d.magnitude() < 0.01);
}

#[test]
fn physics_fallfast() {
    let vol_mgr = ChunkMgr::new(
        CHUNK_SIZE,
        VolGen::new(gen_chunk_flat, gen_payload, drop_chunk, drop_payload),
    );
    vol_mgr.block_loader_mut().push(Arc::new(RwLock::new(BlockLoader {
        pos: Vec3::new(0, 0, 0),
        size: CHUNK_SIZE.map(|e| e as i64 * 10),
    })));
    vol_mgr.gen(Vec3::new(0, 0, 0));
    vol_mgr.gen(Vec3::new(0, 0, 1));
    thread::sleep(time::Duration::from_millis(200)); // because this spawns a thread :/
                                                     //touch
    vol_mgr.maintain();
    let mut ent: HashMap<Uid, Arc<RwLock<Entity<()>>>> = HashMap::new();
    ent.insert(
        1,
        Arc::new(RwLock::new(Entity::new(
            Vec3::new(CHUNK_MID.x, CHUNK_MID.y, CHUNK_SIZE.z as f32 + 10.0),
            Vec3::new(0.0, 0.0, -100.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec2::new(0.0, 0.0),
        ))),
    );
    for _ in 0..100 {
        physics::tick(ent.values(), &vol_mgr, Duration::from_millis(100))
    }
    let p = ent.get(&1);
    let d = *p.unwrap().read().pos() - Vec3::new(CHUNK_MID.x, CHUNK_MID.y, CHUNK_SIZE.z as f32 + 3.0);
    println!("{}, physics_fallfast {}", d.magnitude(), *p.unwrap().read().pos());
    assert!(d.magnitude() < 0.01);
}

#[test]
fn physics_jump() {
    let vol_mgr = ChunkMgr::new(
        CHUNK_SIZE,
        VolGen::new(gen_chunk_flat, gen_payload, drop_chunk, drop_payload),
    );
    vol_mgr.block_loader_mut().push(Arc::new(RwLock::new(BlockLoader {
        pos: Vec3::new(0, 0, 0),
        size: CHUNK_SIZE.map(|e| e as i64 * 10),
    })));
    vol_mgr.gen(Vec3::new(0, 0, 0));
    vol_mgr.gen(Vec3::new(0, 0, 1));
    thread::sleep(time::Duration::from_millis(200)); // because this spawns a thread :/
                                                     //touch
    vol_mgr.maintain();
    let mut ent: HashMap<Uid, Arc<RwLock<Entity<()>>>> = HashMap::new();
    ent.insert(
        1,
        Arc::new(RwLock::new(Entity::new(
            Vec3::new(CHUNK_MID.x, CHUNK_MID.y, CHUNK_SIZE.z as f32 + 10.0),
            Vec3::new(0.0, 0.0, 5.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec2::new(0.0, 0.0),
        ))),
    );
    for _ in 0..3 {
        physics::tick(ent.values(), &vol_mgr, Duration::from_millis(40))
    }
    {
        let p = ent.get(&1);
        assert!(p.unwrap().read().pos().z > 10.2);
    }
    for _ in 0..50 {
        physics::tick(ent.values(), &vol_mgr, Duration::from_millis(100))
    }
    {
        let p = ent.get(&1);
        let d = *p.unwrap().read().pos() - Vec3::new(CHUNK_MID.x, CHUNK_MID.y, CHUNK_SIZE.z as f32 + 3.0);
        //println!("{}", d.magnitude());
        assert!(d.magnitude() < 0.01);
    }
}

#[test]
fn physics_walk() {
    let vol_mgr = ChunkMgr::new(
        CHUNK_SIZE,
        VolGen::new(gen_chunk_flat_border, gen_payload, drop_chunk, drop_payload),
    );
    vol_mgr.block_loader_mut().push(Arc::new(RwLock::new(BlockLoader {
        pos: Vec3::new(0, 0, 0),
        size: CHUNK_SIZE.map(|e| e as i64 * 10),
    })));
    vol_mgr.gen(Vec3::new(0, 0, 0));
    vol_mgr.gen(Vec3::new(0, 0, -1));
    vol_mgr.gen(Vec3::new(1, 0, 0));
    vol_mgr.gen(Vec3::new(1, 0, -1));
    thread::sleep(time::Duration::from_millis(200)); // because this spawns a thread :/
                                                     //touch
    vol_mgr.maintain();
    let mut ent: HashMap<Uid, Arc<RwLock<Entity<()>>>> = HashMap::new();
    ent.insert(
        1,
        Arc::new(RwLock::new(Entity::new(
            Vec3::new(CHUNK_MID.x, CHUNK_MID.y, CHUNK_SIZE.z as f32 + 3.1),
            Vec3::new(3.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec2::new(0.0, 0.0),
        ))),
    );
    for _ in 0..80 {
        physics::tick(ent.values(), &vol_mgr, Duration::from_millis(50))
    }
    {
        let p = ent.get(&1);
        let d = *p.unwrap().read().pos()
            - Vec3::new(
                CHUNK_MID.x*2.0-1.0 - /*player size*/0.45,
                CHUNK_MID.y,
                CHUNK_SIZE.z as f32 + 3.0,
            );
        println!("{}, physics_walk {}", d.magnitude(), *p.unwrap().read().pos());
        // TODO: *DON'T* use chunks below z=0 for these tests, fix this when physics is refactored
        //assert!(d.magnitude() < 0.01);
    }
}
