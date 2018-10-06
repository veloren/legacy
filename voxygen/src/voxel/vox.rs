// Library
use dot_vox::DotVoxData;
use vek::*;

// Project
use common::terrain::{
    figure::{Cell, CellMaterial, Figure},
    Volume, Voxel, ConstructVolume, ReadWriteVolume, ReadVolume,
};

pub fn vox_to_figure(vox: DotVoxData) -> Figure {
    let model = vox.models.first().unwrap();

    let mut figure = Figure::filled(Vec3::new(model.size.x as u16, model.size.y as u16, model.size.z as u16), Cell::empty());
    for ref v in vox.models.first().unwrap().voxels.iter() {
        figure.replace_at(
            Vec3::new(v.x as u16, v.y as u16, v.z as u16),
            Cell::new(CellMaterial::MatteSmooth(v.i)),
        );
    }

    return figure;
}
