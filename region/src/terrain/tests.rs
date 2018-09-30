// Library
use vek::*;

// Local
use super::{raw_chunk::RawChunk, ConstructVolume, ReadVolume, Volume, Voxel, WriteVolume};

#[test]
fn test_raw_chunk() {
    test_volume::<RawChunk>();
    test_read_volume::<RawChunk>();
    test_write_volume::<RawChunk>();
}

fn test_volume<V: Volume + ConstructVolume>() {
    let (sizes, offs) = get_sizes_and_offsets();

    assert!(V::Voxel::empty().is_empty());
    assert!(!V::Voxel::solid().is_empty());

    for sz in sizes {
        let vol = V::empty(sz);

        assert_eq!(vol.get_size(), sz);
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
                assert!(vol.get_at(off).is_some());
                assert!(vol.get_at(off).unwrap().is_empty());
            } else {
                assert!(vol.get_at(off).is_none());
            }
        }

        let vol = V::filled(sz, V::Voxel::solid());

        for off in offs.iter() {
            let off = *off;
            let in_bounds = off.x < sz.x && off.y < sz.y && off.z < sz.z;

            if in_bounds {
                assert!(vol.get_at(off).is_some());
                assert!(!vol.get_at(off).unwrap().is_empty());
            } else {
                assert!(vol.get_at(off).is_none());
            }
        }
    }
}

fn test_write_volume<V: WriteVolume + ConstructVolume>() {
    let (sizes, offs) = get_sizes_and_offsets();

    for sz in sizes {
        for off in offs.iter() {
            let off = *off;
            let in_bounds = off.x < sz.x && off.y < sz.y && off.z < sz.z;

            let mut vol = V::empty(sz);

            let vox = vol.replace_at(off, V::Voxel::solid());

            if in_bounds {
                assert!(vox.unwrap().is_empty());
                assert!(!vol.is_homo());
                assert_eq!(vol.get_at(off), Some(V::Voxel::solid()));
            } else {
                assert!(vol.get_at(off).is_none());
            }

            let vox = vol.replace_at(off, V::Voxel::empty());

            if in_bounds {
                assert!(!vox.unwrap().is_empty());
                assert_eq!(vol.get_at(off), Some(V::Voxel::empty()));
            } else {
                assert!(vol.get_at(off).is_none());
            }
        }
    }
}

fn get_sizes_and_offsets() -> (Vec<Vec3<u64>>, Vec<Vec3<u64>>) {
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
