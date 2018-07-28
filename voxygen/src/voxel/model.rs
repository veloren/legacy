
use gfx;
use gfx::{traits::FactoryExt, Slice, IndexBuffer};
use gfx_device_gl;
use nalgebra::Matrix4;

use voxel::{Mesh, Vertex};
use renderer::{Renderer, ColorFormat, DepthFormat};

gfx_defines! {
    constant Constants {
        model_mat: [[f32; 4]; 4] = "model_mat",
        view_mat: [[f32; 4]; 4] = "view_mat",
        perspective_mat: [[f32; 4]; 4] = "perspective_mat",
    }

    pipeline pipeline {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        constants: gfx::ConstantBuffer<Constants> = "constants",
        out_color: gfx::RenderTarget<ColorFormat> = "target",
        out_depth: gfx::DepthTarget<DepthFormat> = gfx::preset::depth::LESS_EQUAL_WRITE,
    }
}

type PipelineData = pipeline::Data<gfx_device_gl::Resources>;

fn mat4_to_array(mat: &Matrix4<f32>) -> [[f32; 4]; 4] {
    let s = mat.as_slice();
    [
        [s[0],  s[1],  s[2],  s[3]],
        [s[4],  s[5],  s[6],  s[7]],
        [s[8],  s[9],  s[10], s[11]],
        [s[12], s[13], s[14], s[15]],
    ]
}

impl Constants {
    pub fn new(model_mat: &Matrix4<f32>, view_mat: &Matrix4<f32>, perspective_mat: &Matrix4<f32>) -> Constants {
        Constants {
            model_mat: mat4_to_array(&model_mat),
            view_mat: mat4_to_array(&view_mat),
            perspective_mat: mat4_to_array(&perspective_mat),
        }
    }
}

type VertexBuffer = gfx::handle::Buffer<gfx_device_gl::Resources, Vertex>;
type ConstantBuffer = gfx::handle::Buffer<gfx_device_gl::Resources, Constants>;

pub struct Model {
    vbuf: VertexBuffer,
    constants: ConstantBuffer,
    vert_count: u32,
}

impl Model {
    pub fn new(renderer: &mut Renderer, mesh: &Mesh) -> Model {
        Model {
            vbuf: renderer.factory_mut().create_vertex_buffer(&mesh.vertices()),
            constants: renderer.factory_mut().create_constant_buffer(1),
            vert_count: mesh.vert_count(),
        }
    }

    pub fn get_pipeline_data(&self, renderer: &mut Renderer) -> PipelineData {
        PipelineData {
            vbuf: self.vbuf.clone(),
            constants: self.constants.clone(),
            out_color: renderer.color_view().clone(),
            out_depth: renderer.depth_view().clone(),
        }
    }

    pub fn update(&self, renderer: &mut Renderer, constants: Constants) {
        renderer.encoder_mut().update_buffer(&self.constants, &[constants], 0).unwrap();
    }

    pub fn slice(&self) -> Slice<gfx_device_gl::Resources> {
        Slice::<gfx_device_gl::Resources> {
            start: 0,
            end: self.vert_count,
            base_vertex: 0,
            instances: None,
            buffer: IndexBuffer::Auto,
        }
    }
}
