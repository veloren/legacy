// Library
use dot_vox::DotVoxData;
use vek::*;

// Project
use common::terrain::{
    chunk::Block,
    figure::{Cell, CellMaterial, Figure},
    ConstructVolume, ReadWriteVolume, VoxRel, Voxel,
};

pub fn vox_to_figure(vox: DotVoxData) -> Figure {
    let model = vox.models.first().unwrap();

    let mut figure = Figure::empty(Vec3::new(
        model.size.x as VoxRel,
        model.size.y as VoxRel,
        model.size.z as VoxRel,
    ));
    for ref v in vox.models.first().unwrap().voxels.iter() {
        figure.set_at(
            Vec3::new(v.x as VoxRel, v.y as VoxRel, v.z as VoxRel),
            Block::from_byte(v.i),
        );
    }

    return figure;
}
