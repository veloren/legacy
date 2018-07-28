// Library
use coord::prelude::*;
use dot_vox::DotVoxData;

// Project
use region::{Figure, Cell, Voxel};
use client::Volume;

pub fn vox_to_figure(vox: DotVoxData) -> Figure {
    let mut figure = Figure::new();

    let model = vox.models.first().unwrap();
    figure.set_size(vec3!(model.size.x as i64, model.size.y as i64, model.size.z as i64));
    figure.set_offset(vec3!(0,0,0));
    figure.set_scale(vec3!(0.1,0.1,0.1));
    figure.fill(Cell::new(255));
    for ref v in vox.models.first().unwrap().voxels.iter() {
        figure.set(
            vec3!(v.x as i64, v.y as i64, v.z as i64),
            Cell::new(v.i)
        );
    }

    return figure;
}
