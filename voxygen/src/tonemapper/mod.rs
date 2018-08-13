use gfx::{
    self,
    traits::{FactoryExt, Pod},
    IndexBuffer, Slice,
};
use gfx_device_gl;

use consts::{ConstHandle, GlobalConsts};
use renderer::{Renderer, ColorFormat};
use pipeline::Pipeline;

pub type PipelineData = pipeline::Data<gfx_device_gl::Resources>;

gfx_defines! {
    pipeline pipeline {
        in_hdr: gfx::TextureSampler<[f32; 3]> = "t_Hdr",
        global_consts: gfx::ConstantBuffer<GlobalConsts> = "global_consts",
        out_color: gfx::RenderTarget<ColorFormat> = "target",
    }
}

pub fn render(
    renderer: &mut Renderer,
    pipeline: &Pipeline<pipeline::Init<'static>>,
    global_consts: &ConstHandle<GlobalConsts>,
) {
    let data = PipelineData {
        in_hdr: (renderer.hdr_shader_view().clone(), renderer.hdr_sampler().clone()),
        global_consts: global_consts.buffer().clone(),
        out_color: renderer.color_view().clone(),
    };
    let slice = Slice::<gfx_device_gl::Resources> {
        start: 0,
        end: 3,
        base_vertex: 0,
        instances: None,
        buffer: IndexBuffer::Auto,
    };
    renderer.encoder_mut().draw(&slice, pipeline.pso(), &data);
}
