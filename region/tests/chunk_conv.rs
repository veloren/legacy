#![feature(test)]

extern crate region;
extern crate test;
extern crate vek;

// Standard
use std::any::Any;

// Library
use test::Bencher;
use vek::*;

// Project
use region::{
    chunk::{Block, BlockMaterial, BlockRle, Chunk, ChunkContainer, ChunkConverter, ChunkRle},
    Container, PersState, VolContainer, VolConverter, VolPers, Volume, Voxel,
};

/* Reference Chunk

^
|
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

fn gen_raw() -> Chunk {
    let mut result = Chunk::new();
    result.set_size(Vec3::new(4, 4, 4));
    result.fill(Block::empty());
    result.set(Vec3::new(0, 0, 0), Block::new(BlockMaterial::Stone));
    result.set(Vec3::new(0, 1, 0), Block::new(BlockMaterial::Earth));
    result.set(Vec3::new(0, 2, 0), Block::new(BlockMaterial::Stone));
    result.set(Vec3::new(0, 3, 0), Block::new(BlockMaterial::Stone));
    result.set(Vec3::new(1, 0, 0), Block::new(BlockMaterial::Stone));
    result.set(Vec3::new(1, 1, 0), Block::new(BlockMaterial::Earth));
    result.set(Vec3::new(1, 2, 0), Block::new(BlockMaterial::Earth));
    result.set(Vec3::new(1, 3, 0), Block::new(BlockMaterial::Stone));
    result.set(Vec3::new(2, 1, 0), Block::new(BlockMaterial::Earth));
    result.set(Vec3::new(2, 2, 0), Block::new(BlockMaterial::Stone));
    result.set(Vec3::new(2, 3, 0), Block::new(BlockMaterial::Sand));
    result.set(Vec3::new(3, 3, 0), Block::new(BlockMaterial::Stone));

    result.set(Vec3::new(0, 0, 1), Block::new(BlockMaterial::Stone));
    result.set(Vec3::new(0, 1, 1), Block::new(BlockMaterial::Earth));
    result.set(Vec3::new(0, 2, 1), Block::new(BlockMaterial::Earth));
    result.set(Vec3::new(0, 3, 1), Block::new(BlockMaterial::Stone));
    result.set(Vec3::new(1, 2, 1), Block::new(BlockMaterial::Earth));
    result.set(Vec3::new(1, 3, 1), Block::new(BlockMaterial::Sand));
    result.set(Vec3::new(2, 3, 1), Block::new(BlockMaterial::Sand));

    result.set(Vec3::new(0, 0, 2), Block::new(BlockMaterial::Stone));
    result.set(Vec3::new(0, 2, 2), Block::new(BlockMaterial::Earth));
    result.set(Vec3::new(0, 3, 2), Block::new(BlockMaterial::Earth));
    result.set(Vec3::new(1, 0, 2), Block::new(BlockMaterial::Earth));
    result.set(Vec3::new(1, 3, 2), Block::new(BlockMaterial::Sand));
    result.set(Vec3::new(2, 3, 2), Block::new(BlockMaterial::Sand));

    result.set(Vec3::new(0, 0, 3), Block::new(BlockMaterial::Stone));
    result.set(Vec3::new(0, 1, 3), Block::new(BlockMaterial::Earth));
    result.set(Vec3::new(0, 2, 3), Block::new(BlockMaterial::Earth));
    result.set(Vec3::new(1, 0, 3), Block::new(BlockMaterial::Earth));
    result.set(Vec3::new(1, 1, 3), Block::new(BlockMaterial::Earth));
    result.set(Vec3::new(1, 3, 3), Block::new(BlockMaterial::Sand));
    result.set(Vec3::new(2, 0, 3), Block::new(BlockMaterial::Sand));
    result.set(Vec3::new(2, 3, 3), Block::new(BlockMaterial::Sand));
    result.set(Vec3::new(3, 1, 3), Block::new(BlockMaterial::Earth));
    result.set(Vec3::new(3, 2, 3), Block::new(BlockMaterial::Earth));

    return result;
}

fn gen_rle() -> ChunkRle {
    let mut result = ChunkRle::new();
    result.set_size(Vec3::new(4, 4, 4));
    result.fill(Block::empty());
    let dummy = BlockRle::new(Block::new(BlockMaterial::Air), 3);
    {
        let ref mut voxels = result.voxels_mut();
        voxels[0 * 4 + 0].resize(1, dummy);
        voxels[0 * 4 + 0][0] = BlockRle::new(Block::new(BlockMaterial::Stone), 3);
        voxels[0 * 4 + 1].resize(3, dummy);
        voxels[0 * 4 + 1][0] = BlockRle::new(Block::new(BlockMaterial::Earth), 1);
        voxels[0 * 4 + 1][1] = BlockRle::new(Block::new(BlockMaterial::Air), 0);
        voxels[0 * 4 + 1][2] = BlockRle::new(Block::new(BlockMaterial::Earth), 0);
        voxels[0 * 4 + 2].resize(2, dummy);
        voxels[0 * 4 + 2][0] = BlockRle::new(Block::new(BlockMaterial::Stone), 0);
        voxels[0 * 4 + 2][1] = BlockRle::new(Block::new(BlockMaterial::Earth), 2);
        voxels[0 * 4 + 3].resize(2, dummy);
        voxels[0 * 4 + 3][0] = BlockRle::new(Block::new(BlockMaterial::Stone), 1);
        voxels[0 * 4 + 3][1] = BlockRle::new(Block::new(BlockMaterial::Earth), 0);

        voxels[1 * 4 + 0].resize(3, dummy);
        voxels[1 * 4 + 0][0] = BlockRle::new(Block::new(BlockMaterial::Stone), 0);
        voxels[1 * 4 + 0][1] = BlockRle::new(Block::new(BlockMaterial::Air), 0);
        voxels[1 * 4 + 0][2] = BlockRle::new(Block::new(BlockMaterial::Earth), 1);
        voxels[1 * 4 + 1].resize(3, dummy);
        voxels[1 * 4 + 1][0] = BlockRle::new(Block::new(BlockMaterial::Earth), 0);
        voxels[1 * 4 + 1][1] = BlockRle::new(Block::new(BlockMaterial::Air), 1);
        voxels[1 * 4 + 1][2] = BlockRle::new(Block::new(BlockMaterial::Earth), 0);
        voxels[1 * 4 + 2].resize(1, dummy);
        voxels[1 * 4 + 2][0] = BlockRle::new(Block::new(BlockMaterial::Earth), 1);
        voxels[1 * 4 + 3].resize(2, dummy);
        voxels[1 * 4 + 3][0] = BlockRle::new(Block::new(BlockMaterial::Stone), 0);
        voxels[1 * 4 + 3][1] = BlockRle::new(Block::new(BlockMaterial::Sand), 2);

        voxels[2 * 4 + 0].resize(2, dummy);
        voxels[2 * 4 + 0][0] = BlockRle::new(Block::new(BlockMaterial::Air), 2);
        voxels[2 * 4 + 0][1] = BlockRle::new(Block::new(BlockMaterial::Sand), 0);
        voxels[2 * 4 + 1].resize(1, dummy);
        voxels[2 * 4 + 1][0] = BlockRle::new(Block::new(BlockMaterial::Earth), 0);
        voxels[2 * 4 + 2].resize(1, dummy);
        voxels[2 * 4 + 2][0] = BlockRle::new(Block::new(BlockMaterial::Stone), 0);
        voxels[2 * 4 + 3].resize(1, dummy);
        voxels[2 * 4 + 3][0] = BlockRle::new(Block::new(BlockMaterial::Sand), 3);

        voxels[3 * 4 + 0].resize(0, dummy);
        voxels[3 * 4 + 1].resize(2, dummy);
        voxels[3 * 4 + 1][0] = BlockRle::new(Block::new(BlockMaterial::Air), 2);
        voxels[3 * 4 + 1][1] = BlockRle::new(Block::new(BlockMaterial::Earth), 0);
        voxels[3 * 4 + 2].resize(2, dummy);
        voxels[3 * 4 + 2][0] = BlockRle::new(Block::new(BlockMaterial::Air), 2);
        voxels[3 * 4 + 2][1] = BlockRle::new(Block::new(BlockMaterial::Earth), 0);
        voxels[3 * 4 + 3].resize(1, dummy);
        voxels[3 * 4 + 3][0] = BlockRle::new(Block::new(BlockMaterial::Stone), 0);
    }
    return result;
}

#[test]
fn fill_container() {
    let mut con = ChunkContainer::new();
    con.insert(gen_raw(), PersState::Raw);
    assert!(con.contains(PersState::Raw));
    assert!(!con.contains(PersState::File));
    assert!(!con.contains(PersState::Rle));
}

#[test]
fn convert_raw_to_rle() {
    let mut con = ChunkContainer::new();
    con.insert(gen_raw(), PersState::Raw);
    ChunkConverter::convert(&Vec3::<i64>::new(0, 0, 0), &mut con, PersState::Rle);
    assert!(con.contains(PersState::Raw));
    assert!(!con.contains(PersState::File));
    assert!(con.contains(PersState::Rle));
    let rle = con.get_mut(PersState::Rle).unwrap();
    let rle: &ChunkRle = rle.as_any().downcast_ref::<ChunkRle>().expect("Should be ChunkRle");
    let correct_rle = gen_rle();
    // TODO: Set this test up again
    //assert_eq!(correct_rle, *rle);
}

#[test]
fn convert_rle_to_raw() {
    let mut con = ChunkContainer::new();
    con.insert(gen_rle(), PersState::Rle);
    ChunkConverter::convert(&Vec3::<i64>::new(0, 0, 0), &mut con, PersState::Raw);
    assert!(con.contains(PersState::Raw));
    assert!(!con.contains(PersState::File));
    assert!(con.contains(PersState::Rle));
    let raw = con.get_mut(PersState::Raw).unwrap();
    let raw: &Chunk = raw.as_any().downcast_ref::<Chunk>().expect("Should be Chunk");
    let correct_raw = gen_raw();
    // TODO: Set this test up again
    //assert_eq!(correct_raw, *raw);
}

#[bench]
fn raw_to_rle_speed(b: &mut Bencher) {
    let mut con = ChunkContainer::new();
    con.insert(gen_raw(), PersState::Raw);
    b.iter(|| ChunkConverter::convert(&Vec3::<i64>::new(0, 0, 0), &mut con, PersState::Rle));
}

#[bench]
fn rle_to_raw_speed(b: &mut Bencher) {
    let mut con = ChunkContainer::new();
    con.insert(gen_rle(), PersState::Rle);
    b.iter(|| ChunkConverter::convert(&Vec3::<i64>::new(0, 0, 0), &mut con, PersState::Raw));
}
