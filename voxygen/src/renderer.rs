use gfx::{
    self,
    handle::{DepthStencilView, RenderTargetView},
    Device, Encoder,
};
use gfx_device_gl;

use pipeline::Pipeline;
use shader::Shader;
use voxel;

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
            voxel_pipeline: Pipeline::new(
                &mut factory,
                voxel::pipeline::new(),
                &Shader::from_file("shaders/vert.glsl").expect("Could not load vertex shader"),
                &Shader::from_file("shaders/frag.glsl").expect("Could not load fragment shader"),
            ),
            factory,
        }
    }

    pub fn begin_frame(&mut self) {
        self.encoder.clear(&self.color_view, [0.5, 0.7, 1.0, 1.0]);
        self.encoder.clear_depth(&self.depth_view, 1.0);
    }

    pub fn render_model_object(
        &mut self,
        vmodel: &voxel::Model,
        world_consts: &voxel::ConstHandle<voxel::WorldConsts>,
    ) {
        let pipeline_data = vmodel.get_pipeline_data(self, world_consts);
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
