use gfx::{
    self,
    traits::{FactoryExt, Pod},
    IndexBuffer, Slice,
};
use gfx_device_gl;

use renderer::{ColorFormat, DepthFormat, Renderer};
use voxel::{Mesh, Vertex};

type PipelineData = pipeline::Data<gfx_device_gl::Resources>;
type ConstBuffer<T> = gfx::handle::Buffer<gfx_device_gl::Resources, T>;
type VertexBuffer = gfx::handle::Buffer<gfx_device_gl::Resources, Vertex>;

gfx_defines! {
    constant ModelConsts {
        model_mat: [[f32; 4]; 4] = "model_mat",
    }

    constant WorldConsts {
        view_mat: [[f32; 4]; 4] = "view_mat",
        proj_mat: [[f32; 4]; 4] = "proj_mat",
        sky_color: [f32; 4] = "sky_color",
        play_origin: [f32; 4] = "play_origin",
        view_distance: [f32; 4] = "view_distance",
        time: [f32; 4] = "time",
    }

    pipeline pipeline {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        model_consts: gfx::ConstantBuffer<ModelConsts> = "model_consts",
        world_consts: gfx::ConstantBuffer<WorldConsts> = "world_consts",
        out_color: gfx::RenderTarget<ColorFormat> = "target",
        out_depth: gfx::DepthTarget<DepthFormat> = gfx::preset::depth::LESS_EQUAL_WRITE,
    }
}

pub struct ConstHandle<T: Copy + Pod> {
    buffer: ConstBuffer<T>,
}

impl<T: Copy + Pod> ConstHandle<T> {
    pub fn new(renderer: &mut Renderer) -> ConstHandle<T> {
        ConstHandle {
            buffer: renderer.factory_mut().create_constant_buffer(1),
        }
    }

    pub fn update(&self, renderer: &mut Renderer, consts: T) {
        renderer
            .encoder_mut()
            .update_buffer(&self.buffer, &[consts], 0)
            .unwrap();
    }

    fn buffer(&self) -> &ConstBuffer<T> { &self.buffer }
}

pub struct Model {
    vbuf: VertexBuffer,
    const_handle: ConstHandle<ModelConsts>,
    vert_count: u32,
}

impl Model {
    pub fn new(renderer: &mut Renderer, mesh: &Mesh) -> Model {
        Model {
            vbuf: renderer.factory_mut().create_vertex_buffer(&mesh.vertices()),
            const_handle: ConstHandle::new(renderer),
            vert_count: mesh.vert_count(),
        }
    }

    pub fn const_handle(&self) -> &ConstHandle<ModelConsts> { &self.const_handle }

    pub fn get_pipeline_data(&self, renderer: &mut Renderer, world_consts: &ConstHandle<WorldConsts>) -> PipelineData {
        PipelineData {
            vbuf: self.vbuf.clone(),
            model_consts: self.const_handle.buffer().clone(),
            world_consts: world_consts.buffer().clone(),
            out_color: renderer.color_view().clone(),
            out_depth: renderer.depth_view().clone(),
        }
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
