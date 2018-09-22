// Library
use vek::*;
use gfx;
use gfx::{
    VertexBuffer,
    BlendTarget,
    preset::blend::ALPHA,
    state::{ColorMask, Rasterizer},
    Primitive::TriangleList,
    pso::PipelineInit,
    PipelineState,
    traits::FactoryExt,
};
use gfx_device_gl;
use lyon::{
    tessellation,
    tessellation::geometry_builder::{
        BuffersBuilder,
        VertexBuffers,
        VertexConstructor,
    },
};

// Local
use renderer::ColorFormat;
use renderer::Renderer;
use shader::Shader;

// Vertex

gfx_defines! {
    vertex FillVertex {
        pos: [f32; 2] = "v_pos",
        col: [f32; 4] = "v_col",
    }
}

impl FillVertex {
    pub fn new(pos: Vec2<f32>, col: Rgba<f32>) -> FillVertex {
        FillVertex {
            pos: pos.into_array(),
            col: col.into_array(),
        }
    }
}

// VertexFactory

pub struct VertexFactory {
    col: Rgba<f32>,
}

impl VertexFactory {
    pub fn with_color(col: Rgba<f32>) -> VertexFactory {
        VertexFactory { col }
    }
}

impl VertexConstructor<tessellation::FillVertex, FillVertex> for VertexFactory {
    fn new_vertex(&mut self, vertex: tessellation::FillVertex) -> FillVertex {
        FillVertex::new(vertex.position.to_array().into(), self.col)
    }
}

// fill_pipeline

gfx_defines! {
    pipeline fill_pipeline {
        vbo: VertexBuffer<FillVertex> = (),
        out_color: BlendTarget<ColorFormat> = ("target", ColorMask::all(), ALPHA),
    }
}

pub(crate) type FillPso = PipelineState<gfx_device_gl::Resources, <fill_pipeline::Init<'static> as PipelineInit>::Meta>;

pub fn create_fill_pso(renderer: &mut Renderer) -> FillPso {
    let vs = Shader::from_str("
        #version 140

        in vec2 v_pos;
        in vec4 v_col;
        out vec4 f_col;

        void main() {
            gl_Position = vec4(vec2(2.0, -2.0) * v_pos + vec2(-1.0, 1.0), 0.0, 1.0);
            f_col = v_col;
        }
    ");

    let fs = Shader::from_str("
        #version 140

        in vec4 f_col;
        out vec4 target;

        void main() {
            target = f_col;
        }
    ");

    let program = renderer.factory_mut()
        .link_program(vs.bytes(), fs.bytes())
        .expect("Failed to link fill PSO");

    renderer.factory_mut()
        .create_pipeline_from_program(
            &program,
            TriangleList,
            Rasterizer::new_fill(),
            fill_pipeline::new()
        )
        .expect("Failed to create fill PSO")
}
