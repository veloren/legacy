// Library
use vek::*;
use std::fmt::Debug;

// Local
use terrain::{
    chunk::{Block, BlockMaterial, BlockRle, Chunk, ChunkContainer, HeterogeneousData, RleData, HomogeneousData},
    Container, PersState, VolPers, Volume, Voxel, ConstructVolume, VolCluster, ReadVolume, ReadWriteVolume,
};


#[test]
fn test_hetero_chunk() {
    test_volume::<HeterogeneousData>();
    test_read_volume::<HeterogeneousData>();
    test_write_volume::<HeterogeneousData>();
}

#[test]
fn test_rle_chunk() {
    test_volume::<RleData>();
    test_read_volume::<RleData>();
}

#[test]
fn test_homo_chunk() {
    test_volume::<HomogeneousData>();
    test_read_volume::<HomogeneousData>();
}

fn test_volume<V: Volume + ConstructVolume>() {
    let (sizes, offs) = get_sizes_and_offsets();

    assert!(!V::VoxelType::empty().is_solid());

    for sz in sizes {
        let vol = V::empty(sz);

        assert_eq!(vol.size(), sz);
    }
}

fn test_read_volume<V: ReadVolume + ConstructVolume>() {
    let (sizes, offs) = get_sizes_and_offsets();

    for sz in sizes {
        let vol = V::empty(sz);

        for off in offs.iter() {
            let off = *off;
            let in_bounds = off.x < sz.x && off.y < sz.y && off.z < sz.z;

            if in_bounds {
                assert!(vol.at(off).is_some());
                assert!(!vol.at(off).unwrap().is_solid());
            } else {
                assert!(vol.at(off).is_none());
            }
        }
    }
}

fn test_write_volume<V: ReadWriteVolume + ConstructVolume>() where V::VoxelType: Debug + PartialEq {
    let (sizes, offs) = get_sizes_and_offsets();

    for sz in sizes {
        for off in offs.iter() {
            let off = *off;
            let in_bounds = off.x < sz.x && off.y < sz.y && off.z < sz.z;

            let mut vol = V::empty(sz);

            let vox = vol.replace_at(off, V::VoxelType::empty());

            if in_bounds {
                assert!(!vox.unwrap().is_solid());
                assert_eq!(vol.at(off), Some(V::VoxelType::empty()));
            } else {
                assert!(vol.at(off).is_none());
            }
        }
    }
}

fn get_sizes_and_offsets() -> (Vec<Vec3<u16>>, Vec<Vec3<u16>>) {
    // Volume sizes to perform the tests in
    let sizes = vec![
        Vec3::new(0, 0, 0),
        Vec3::new(0, 1, 1),
        Vec3::new(1, 0, 1),
        Vec3::new(1, 1, 0),
        Vec3::new(1, 1, 1),
        Vec3::new(5, 5, 5),
        Vec3::new(5, 10, 10),
        Vec3::new(10, 5, 10),
        Vec3::new(10, 10, 5),
        Vec3::new(10, 0, 0),
        Vec3::new(0, 10, 0),
        Vec3::new(0, 0, 10),
        Vec3::new(5, 0, 0),
        Vec3::new(0, 5, 0),
        Vec3::new(0, 0, 5),
    ];

    // Volume offsets to perform tests with
    let offs = vec![
        Vec3::new(0, 0, 0),
        Vec3::new(1, 0, 0),
        Vec3::new(0, 1, 0),
        Vec3::new(0, 0, 1),
        Vec3::new(1, 1, 1),
        Vec3::new(4, 4, 4),
        Vec3::new(4, 0, 0),
        Vec3::new(0, 4, 0),
        Vec3::new(0, 0, 4),
        Vec3::new(5, 5, 5),
        Vec3::new(5, 0, 0),
        Vec3::new(0, 5, 0),
        Vec3::new(0, 0, 5),
        Vec3::new(1, 2, 3),
        Vec3::new(3, 2, 1),
        Vec3::new(9, 9, 9),
    ];

    (sizes, offs)
}
