#![feature(test)]

extern crate test;

// Standard

// Library
use test::Bencher;
use vek::*;

// Project
use common::terrain::{
    chunk::{Block, BlockMat, BlockRle, Chunk, HeterogeneousData, RleData},
    ConstructVolume, PersState, ReadWriteVolume, VolCluster, Voxel,
};

/* Reference Chunk

^
|z
y
  x -->

z:0
4424
434
333
44

z:1
422
33
3
4

z:2
322
3
0
43

z:3
022
3003
3303
432

*/

fn gen_hetero() -> HeterogeneousData {
    let mut result = HeterogeneousData::empty(Vec3::new(4, 4, 4));
    result.replace_at_unchecked(Vec3::new(0, 0, 0), Block::STONE);
    result.replace_at_unchecked(Vec3::new(0, 1, 0), Block::EARTH);
    result.replace_at_unchecked(Vec3::new(0, 2, 0), Block::STONE);
    result.replace_at_unchecked(Vec3::new(0, 3, 0), Block::STONE);
    result.replace_at_unchecked(Vec3::new(1, 0, 0), Block::STONE);
    result.replace_at_unchecked(Vec3::new(1, 1, 0), Block::EARTH);
    result.replace_at_unchecked(Vec3::new(1, 2, 0), Block::EARTH);
    result.replace_at_unchecked(Vec3::new(1, 3, 0), Block::STONE);
    result.replace_at_unchecked(Vec3::new(2, 1, 0), Block::EARTH);
    result.replace_at_unchecked(Vec3::new(2, 2, 0), Block::STONE);
    result.replace_at_unchecked(Vec3::new(2, 3, 0), Block::SAND);
    result.replace_at_unchecked(Vec3::new(3, 3, 0), Block::STONE);

    result.replace_at_unchecked(Vec3::new(0, 0, 1), Block::STONE);
    result.replace_at_unchecked(Vec3::new(0, 1, 1), Block::EARTH);
    result.replace_at_unchecked(Vec3::new(0, 2, 1), Block::EARTH);
    result.replace_at_unchecked(Vec3::new(0, 3, 1), Block::STONE);
    result.replace_at_unchecked(Vec3::new(1, 2, 1), Block::EARTH);
    result.replace_at_unchecked(Vec3::new(1, 3, 1), Block::SAND);
    result.replace_at_unchecked(Vec3::new(2, 3, 1), Block::SAND);

    result.replace_at_unchecked(Vec3::new(0, 0, 2), Block::STONE);
    result.replace_at_unchecked(Vec3::new(0, 2, 2), Block::EARTH);
    result.replace_at_unchecked(Vec3::new(0, 3, 2), Block::EARTH);
    result.replace_at_unchecked(Vec3::new(1, 0, 2), Block::EARTH);
    result.replace_at_unchecked(Vec3::new(1, 3, 2), Block::SAND);
    result.replace_at_unchecked(Vec3::new(2, 3, 2), Block::SAND);

    result.replace_at_unchecked(Vec3::new(0, 0, 3), Block::STONE);
    result.replace_at_unchecked(Vec3::new(0, 1, 3), Block::EARTH);
    result.replace_at_unchecked(Vec3::new(0, 2, 3), Block::EARTH);
    result.replace_at_unchecked(Vec3::new(1, 0, 3), Block::EARTH);
    result.replace_at_unchecked(Vec3::new(1, 1, 3), Block::EARTH);
    result.replace_at_unchecked(Vec3::new(1, 3, 3), Block::SAND);
    result.replace_at_unchecked(Vec3::new(2, 0, 3), Block::SAND);
    result.replace_at_unchecked(Vec3::new(2, 3, 3), Block::SAND);
    result.replace_at_unchecked(Vec3::new(3, 1, 3), Block::EARTH);
    result.replace_at_unchecked(Vec3::new(3, 2, 3), Block::EARTH);

    return result;
}

fn gen_rle() -> RleData {
    let mut result = RleData::empty(Vec3::new(4, 4, 4));
    let dummy = BlockRle::new(Block::AIR, 3);
    {
        let ref mut voxels = result.voxels_mut_internal();
        voxels[0 * 4 + 0].resize(1, dummy);
        voxels[0 * 4 + 0][0] = BlockRle::new(Block::STONE, 3);
        voxels[0 * 4 + 1].resize(3, dummy);
        voxels[0 * 4 + 1][0] = BlockRle::new(Block::EARTH, 1);
        voxels[0 * 4 + 1][1] = BlockRle::new(Block::AIR, 0);
        voxels[0 * 4 + 1][2] = BlockRle::new(Block::EARTH, 0);
        voxels[0 * 4 + 2].resize(2, dummy);
        voxels[0 * 4 + 2][0] = BlockRle::new(Block::STONE, 0);
        voxels[0 * 4 + 2][1] = BlockRle::new(Block::EARTH, 2);
        voxels[0 * 4 + 3].resize(2, dummy);
        voxels[0 * 4 + 3][0] = BlockRle::new(Block::STONE, 1);
        voxels[0 * 4 + 3][1] = BlockRle::new(Block::EARTH, 0);

        voxels[1 * 4 + 0].resize(3, dummy);
        voxels[1 * 4 + 0][0] = BlockRle::new(Block::STONE, 0);
        voxels[1 * 4 + 0][1] = BlockRle::new(Block::AIR, 0);
        voxels[1 * 4 + 0][2] = BlockRle::new(Block::EARTH, 1);
        voxels[1 * 4 + 1].resize(3, dummy);
        voxels[1 * 4 + 1][0] = BlockRle::new(Block::EARTH, 0);
        voxels[1 * 4 + 1][1] = BlockRle::new(Block::AIR, 1);
        voxels[1 * 4 + 1][2] = BlockRle::new(Block::EARTH, 0);
        voxels[1 * 4 + 2].resize(1, dummy);
        voxels[1 * 4 + 2][0] = BlockRle::new(Block::EARTH, 1);
        voxels[1 * 4 + 3].resize(2, dummy);
        voxels[1 * 4 + 3][0] = BlockRle::new(Block::STONE, 0);
        voxels[1 * 4 + 3][1] = BlockRle::new(Block::SAND, 2);

        voxels[2 * 4 + 0].resize(2, dummy);
        voxels[2 * 4 + 0][0] = BlockRle::new(Block::AIR, 2);
        voxels[2 * 4 + 0][1] = BlockRle::new(Block::SAND, 0);
        voxels[2 * 4 + 1].resize(1, dummy);
        voxels[2 * 4 + 1][0] = BlockRle::new(Block::EARTH, 0);
        voxels[2 * 4 + 2].resize(1, dummy);
        voxels[2 * 4 + 2][0] = BlockRle::new(Block::STONE, 0);
        voxels[2 * 4 + 3].resize(1, dummy);
        voxels[2 * 4 + 3][0] = BlockRle::new(Block::SAND, 3);

        voxels[3 * 4 + 0].resize(0, dummy);
        voxels[3 * 4 + 1].resize(2, dummy);
        voxels[3 * 4 + 1][0] = BlockRle::new(Block::AIR, 2);
        voxels[3 * 4 + 1][1] = BlockRle::new(Block::EARTH, 0);
        voxels[3 * 4 + 2].resize(2, dummy);
        voxels[3 * 4 + 2][0] = BlockRle::new(Block::AIR, 2);
        voxels[3 * 4 + 2][1] = BlockRle::new(Block::EARTH, 0);
        voxels[3 * 4 + 3].resize(1, dummy);
        voxels[3 * 4 + 3][0] = BlockRle::new(Block::STONE, 0);
    }
    return result;
}

#[test]
fn fill_container() {
    let con = Chunk::Hetero(gen_hetero());
    assert!(con.contains(PersState::Hetero));
    assert!(!con.contains(PersState::Homo));
    assert!(!con.contains(PersState::Rle));
}

#[test]
fn convert_raw_to_rle() {
    let mut con = Chunk::Hetero(gen_hetero());
    con.convert(PersState::Rle);
    assert!(con.contains(PersState::Hetero));
    assert!(!con.contains(PersState::Homo));
    assert!(con.contains(PersState::Rle));
    let rle = con.get_any(PersState::Rle).unwrap();
    let rle: &RleData = rle.as_any().downcast_ref::<RleData>().expect("Should be RleData");
    let correct_rle = gen_rle();
    assert_eq!(correct_rle, *rle);
}

#[test]
fn convert_rle_to_raw() {
    let mut con = Chunk::Rle(gen_rle());
    con.convert(PersState::Hetero);
    assert!(con.contains(PersState::Hetero));
    assert!(!con.contains(PersState::Homo));
    assert!(con.contains(PersState::Rle));
    let hetero = con.get_any(PersState::Hetero).unwrap();
    let hetero: &HeterogeneousData = hetero
        .as_any()
        .downcast_ref::<HeterogeneousData>()
        .expect("Should be HeterogeneousData");
    let correct_hetero = gen_hetero();
    // TODO: Set this test up again
    assert_eq!(correct_hetero, *hetero);
}

#[test]
fn read_rle() {
    let con = Chunk::Rle(gen_rle());
    let access = con.prefered().unwrap();
    assert_eq!(access.at(Vec3::new(0, 0, 0)), Some(Block::STONE));
    assert_eq!(access.at(Vec3::new(0, 1, 0)), Some(Block::EARTH));
    assert_eq!(access.at(Vec3::new(0, 2, 0)), Some(Block::STONE));
    assert_eq!(access.at(Vec3::new(0, 3, 0)), Some(Block::STONE));
    assert_eq!(access.at(Vec3::new(0, 4, 0)), None);

    assert_eq!(access.at(Vec3::new(1, 0, 3)), Some(Block::EARTH));
    assert_eq!(access.at(Vec3::new(1, 1, 3)), Some(Block::EARTH));
    assert_eq!(access.at(Vec3::new(1, 2, 3)), Some(Block::AIR));
    assert_eq!(access.at(Vec3::new(1, 3, 3)), Some(Block::SAND));

    assert_eq!(access.at(Vec3::new(2, 2, 0)), Some(Block::STONE));
    assert_eq!(access.at(Vec3::new(2, 2, 1)), Some(Block::AIR));
    assert_eq!(access.at(Vec3::new(2, 2, 2)), Some(Block::AIR));
    assert_eq!(access.at(Vec3::new(2, 2, 3)), Some(Block::AIR));

    assert_eq!(access.at(Vec3::new(0, 3, 0)), Some(Block::STONE));
    assert_eq!(access.at(Vec3::new(0, 3, 1)), Some(Block::STONE));
    assert_eq!(access.at(Vec3::new(0, 3, 2)), Some(Block::EARTH));
    assert_eq!(access.at(Vec3::new(0, 3, 3)), Some(Block::AIR));
}

#[bench]
fn raw_to_rle_speed(b: &mut Bencher) {
    b.iter(|| {
        let mut con = Chunk::Hetero(gen_hetero());
        con.convert(PersState::Rle);
    });
}

#[bench]
fn rle_to_raw_speed(b: &mut Bencher) {
    b.iter(|| {
        let mut con = Chunk::Rle(gen_rle());
        con.convert(PersState::Hetero);
    });
}
