// Library
use vek::*;
use gfx;
use gfx::{
    VertexBuffer,
    BlendTarget,
    preset::blend::ALPHA,
    state::ColorMask,
};
use lyon::tessellation::{
    FillVertex,
    geometry_builder::{
        BuffersBuilder, VertexBuffers, VertexConstructor,
    },
};

// Local
use renderer::ColorFormat;

// Vertex

gfx_defines! {
    vertex Vertex {
        pos: [f32; 2] = "v_pos",
        col: [f32; 4] = "v_col",
    }
}

impl Vertex {
    pub fn new(pos: Vec2<f32>, col: Rgba<f32>) -> Vertex {
        Vertex {
            pos: pos.into_array(),
            col: col.into_array(),
        }
    }
}

// VertexFactory

pub struct VertexFactory {
    col: Rgba<f32>,
}

impl VertexConstructor<FillVertex, Vertex> for VertexFactory {
    fn new_vertex(&mut self, vertex: FillVertex) -> Vertex {
        Vertex::new(vertex.position.to_array().into(), self.col)
    }
}

// fill_pipeline

gfx_defines! {
    pipeline fill_pipeline {
        vbo: VertexBuffer<Vertex> = (),
        out_color: BlendTarget<ColorFormat> = ("target", ColorMask::all(), ALPHA),
    }
}
