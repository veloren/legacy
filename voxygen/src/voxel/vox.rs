// Standard

// Library
use coord::prelude::*;
use dot_vox;

// Project
use region::{Model, Cell, Voxel};
use client::Volume;

pub fn vox_to_model(vox: dot_vox::DotVoxData) -> Model {
    let model = vox.models.first().unwrap();
    let mut chunk = Model::new();
    chunk.set_size(vec3!(model.size.x as i64, model.size.y as i64, model.size.z as i64));
    chunk.set_offset(vec3!(0,0,0));
    chunk.set_scale(vec3!(0.1,0.1,0.1));
    chunk.fill(Cell::new(255));
    for ref v in model.voxels.iter() {
        chunk.set(
            vec3!(v.x as i64, v.y as i64, v.z as i64),
            Cell::new(v.i)
        );
    }

    return chunk;
}
