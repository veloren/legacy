// Library
use coord::prelude::*;
use dot_vox::DotVoxData;

// Project
use client::Volume;
use region::{Cell, CellMaterial, Figure, Voxel};

pub fn vox_to_figure(vox: DotVoxData) -> Figure {
    let mut figure = Figure::new();

    let model = vox.models.first().unwrap();
    figure.set_size(vec3!(model.size.x as i64, model.size.y as i64, model.size.z as i64));
    figure.set_offset(vec3!(0, 0, 0));
    figure.set_scale(vec3!(0.1, 0.1, 0.1));
    figure.fill(Cell::new(CellMaterial::Empty));
    for ref v in vox.models.first().unwrap().voxels.iter() {
        figure.set(
            vec3!(v.x as i64, v.y as i64, v.z as i64),
            Cell::new(CellMaterial::MatteSmooth(v.i)),
        );
    }

    return figure;
}
