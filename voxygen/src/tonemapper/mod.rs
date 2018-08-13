use gfx::{
    self,
    traits::{FactoryExt, Pod},
    IndexBuffer, Slice,
};
use gfx_device_gl;

use consts::{GlobalConsts};
use renderer::{ColorFormat};

pub type PipelineData = pipeline::Data<gfx_device_gl::Resources>;

gfx_defines! {
    pipeline pipeline {
        in_hdr: gfx::TextureSampler<[f32; 3]> = "t_Hdr",
        global_consts: gfx::ConstantBuffer<GlobalConsts> = "global_consts",
        out_color: gfx::RenderTarget<ColorFormat> = "target",
    }
}