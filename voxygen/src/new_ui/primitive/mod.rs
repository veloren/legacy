// Standard
use std::hash::Hash;

// Library
use vek::*;

// Library
use lyon::{
    math::rect,
    tessellation::{
        FillOptions,
        basic_shapes::fill_rectangle,
        geometry_builder::{BuffersBuilder, VertexBuffers},
    },
};
use gfx::traits::FactoryExt;

// Local
use renderer::Renderer;
use shader::Shader;
use super::{
    ResCache,
    render::{create_fill_pso, fill_pipeline, FillVertex, VertexFactory},
};

fn create_rect_mesh(pos: Vec2<f32>, sz: Vec2<f32>, col: Rgba<f32>) -> VertexBuffers<FillVertex, u16> {
    let mut mesh: VertexBuffers<FillVertex, u16> = VertexBuffers::new();

    fill_rectangle(
        &rect(pos.x, pos.y, sz.x, sz.y),
        &FillOptions::tolerance(0.0001),
        &mut BuffersBuilder::new(&mut mesh, VertexFactory::with_color(col)),
    );

    mesh
}

pub(crate) fn draw_rectangle(renderer: &mut Renderer, rescache: &mut ResCache, pos: Vec2<f32>, sz: Vec2<f32>, col: Rgba<f32>) {
    let mesh = rescache.get_or_create_rect_mesh(pos, sz, col, || create_rect_mesh(pos, sz, col));
    let pso = rescache.get_or_create_fill_pso(|| create_fill_pso(renderer));

    let (buffer, slice) = renderer.factory_mut()
        .create_vertex_buffer_with_slice(&mesh.vertices[..], &mesh.indices[..]);

    let color_view = renderer.color_view().clone();

    renderer.encoder_mut().draw(
        &slice,
        &pso,
        &fill_pipeline::Data {
            vbo: buffer,
            out_color: color_view,
        },
    );
}
