use coord::prelude::*;
use gfx::{
    self,
    handle::{DepthStencilView, RenderTargetView},
    Device, Encoder,
};
use gfx_device_gl;

use consts::{ConstHandle, GlobalConsts};
use pipeline::Pipeline;
use shader::Shader;
use voxel;
use skybox;

pub type ColorFormat = gfx::format::Srgba8;
pub type DepthFormat = gfx::format::DepthStencil;

pub type ColorView = RenderTargetView<gfx_device_gl::Resources, ColorFormat>;
pub type DepthView = DepthStencilView<gfx_device_gl::Resources, DepthFormat>;

pub struct Renderer {
    device: gfx_device_gl::Device,
    color_view: ColorView,
    depth_view: DepthView,
    factory: gfx_device_gl::Factory,
    encoder: Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>,
    skybox_pipeline: Pipeline<skybox::pipeline::Init<'static>>,
    voxel_pipeline: Pipeline<voxel::pipeline::Init<'static>>,
}

impl Renderer {
    pub fn new(
        device: gfx_device_gl::Device,
        mut factory: gfx_device_gl::Factory,
        color_view: ColorView,
        depth_view: DepthView,
    ) -> Renderer {
        Renderer {
            device,
            color_view,
            depth_view,
            encoder: factory.create_command_buffer().into(),
            skybox_pipeline: Pipeline::new(
                &mut factory,
                skybox::pipeline::new(),
                &Shader::from_file("shaders/skybox/vert.glsl").expect("Could not load skybox vertex shader"),
                &Shader::from_file("shaders/skybox/frag.glsl").expect("Could not load skybox fragment shader"),
            ),
            voxel_pipeline: Pipeline::new(
                &mut factory,
                voxel::pipeline::new(),
                &Shader::from_file("shaders/voxel/vert.glsl").expect("Could not load voxel vertex shader"),
                &Shader::from_file("shaders/voxel/frag.glsl").expect("Could not load voxel fragment shader"),
            ),
            factory,
        }
    }

    pub fn begin_frame(&mut self, clear_color: Vec3<f32>) {
        self.encoder
            .clear(&self.color_view, [clear_color.x, clear_color.y, clear_color.z, 1.0]);
        self.encoder.clear_depth(&self.depth_view, 1.0);
    }

    pub fn render_skybox_model(
        &mut self,
        vmodel: &skybox::Model,
        global_consts: &ConstHandle<GlobalConsts>,
    ) {
        let pipeline_data = vmodel.get_pipeline_data(self, global_consts);
        self.encoder
            .draw(&vmodel.slice(), self.skybox_pipeline.pso(), &pipeline_data);
    }

    pub fn render_voxel_model(
        &mut self,
        vmodel: &voxel::Model,
        global_consts: &ConstHandle<GlobalConsts>,
    ) {
        let pipeline_data = vmodel.get_pipeline_data(self, global_consts);
        self.encoder
            .draw(&vmodel.slice(), self.voxel_pipeline.pso(), &pipeline_data);
    }

    pub fn end_frame(&mut self) {
        self.encoder.flush(&mut self.device);
        self.device.cleanup();
    }

    #[allow(dead_code)]
    pub fn encoder_mut(&mut self) -> &mut Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer> {
        &mut self.encoder
    }
    #[allow(dead_code)]
    pub fn factory_mut(&mut self) -> &mut gfx_device_gl::Factory { &mut self.factory }
    #[allow(dead_code)]
    pub fn color_view(&self) -> &ColorView { &self.color_view }
    #[allow(dead_code)]
    pub fn depth_view(&self) -> &DepthView { &self.depth_view }

    #[allow(dead_code)]
    pub fn set_views(&mut self, color_view: ColorView, depth_view: DepthView) {
        self.color_view = color_view;
        self.depth_view = depth_view;
    }
}
