use gfx::{
    self,
    traits::{FactoryExt, Pod},
    IndexBuffer, Slice,
};
use gfx_device_gl;

use consts::{ConstHandle, GlobalConsts};
use pipeline::Pipeline;
use renderer::{HdrFormat, HdrDepthFormat, Renderer};
use voxel::{Mesh, Vertex};

type PipelineData = pipeline::Data<gfx_device_gl::Resources>;
type VertexBuffer = gfx::handle::Buffer<gfx_device_gl::Resources, Vertex>;

gfx_defines! {
    constant ModelConsts {
        model_mat: [[f32; 4]; 4] = "model_mat",
    }

    pipeline pipeline {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        model_consts: gfx::ConstantBuffer<ModelConsts> = "model_consts",
        global_consts: gfx::ConstantBuffer<GlobalConsts> = "global_consts",
        out_color: gfx::RenderTarget<HdrFormat> = "target",
        out_depth: gfx::DepthTarget<HdrDepthFormat> = gfx::preset::depth::LESS_EQUAL_WRITE,
    }
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

    pub fn get_pipeline_data(
        &self,
        renderer: &mut Renderer,
        global_consts: &ConstHandle<GlobalConsts>,
    ) -> PipelineData {
        PipelineData {
            vbuf: self.vbuf.clone(),
            model_consts: self.const_handle.buffer().clone(),
            global_consts: global_consts.buffer().clone(),
            out_color: renderer.hdr_render_view().clone(),
            out_depth: renderer.hdr_depth_view().clone(),
        }
    }

    pub fn slice(&self) -> Slice<gfx_device_gl::Resources> {
        // TODO: Should we be recreating this every time we render it? Is there a cost associated?
        Slice::<gfx_device_gl::Resources> {
            start: 0,
            end: self.vert_count,
            base_vertex: 0,
            instances: None,
            buffer: IndexBuffer::Auto,
        }
    }

    pub fn render(
        &self,
        renderer: &mut Renderer,
        pipeline: &Pipeline<pipeline::Init<'static>>,
        global_consts: &ConstHandle<GlobalConsts>,
    ) {
        let pipeline_data = self.get_pipeline_data(renderer, global_consts);
        renderer
            .encoder_mut()
            .draw(&self.slice(), pipeline.pso(), &pipeline_data);
    }
}
