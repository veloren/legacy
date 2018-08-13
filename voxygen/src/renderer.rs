use coord::prelude::*;
use gfx::{
    self,
    handle::{DepthStencilView, RenderTargetView, ShaderResourceView, Sampler},
    texture::{SamplerInfo, FilterMethod, WrapMode},
    format::Formatted,
    Device, Encoder, Factory,
    Slice, IndexBuffer,
};
use gfx_device_gl;

use consts::{ConstHandle, GlobalConsts};
use pipeline::Pipeline;
use shader::Shader;
use voxel;
use skybox;
use tonemapper;

pub type HdrFormat = (gfx::format::R32_G32_B32, gfx::format::Float);
pub type ColorFormat = gfx::format::Srgba8;
pub type DepthFormat = gfx::format::DepthStencil;
pub type HdrDepthFormat = gfx::format::Depth32F;

pub type ColorView = RenderTargetView<gfx_device_gl::Resources, ColorFormat>;
pub type DepthView = DepthStencilView<gfx_device_gl::Resources, DepthFormat>;
pub type HdrDepthView = DepthStencilView<gfx_device_gl::Resources, HdrDepthFormat>;

pub type HdrShaderView = ShaderResourceView<gfx_device_gl::Resources, <HdrFormat as Formatted>::View>;
pub type HdrRenderView = RenderTargetView<gfx_device_gl::Resources, HdrFormat>;


pub struct Renderer {
    device: gfx_device_gl::Device,
    color_view: ColorView,
    depth_view: DepthView,
    hdr_shader_view: HdrShaderView,
    hdr_render_view: HdrRenderView,
    hdr_depth_view: HdrDepthView,
    hdr_sampler: Sampler<gfx_device_gl::Resources>,
    factory: gfx_device_gl::Factory,
    encoder: Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>,
    skybox_pipeline: Pipeline<skybox::pipeline::Init<'static>>,
    voxel_pipeline: Pipeline<voxel::pipeline::Init<'static>>,
    tonemapper_pipeline: Pipeline<tonemapper::pipeline::Init<'static>>,
}

impl Renderer {
    pub fn new(
        device: gfx_device_gl::Device,
        mut factory: gfx_device_gl::Factory,
        color_view: ColorView,
        depth_view: DepthView,
        size: (u16, u16),
    ) -> Renderer {
        let (hdr_shader_view, hdr_render_view, hdr_depth_view, hdr_sampler) = Self::create_hdr_views(&mut factory, size);
        Renderer {
            device,
            color_view,
            depth_view,
            hdr_shader_view,
            hdr_render_view,
            hdr_depth_view,
            hdr_sampler,
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
            tonemapper_pipeline: Pipeline::new(
                &mut factory,
                tonemapper::pipeline::new(),
                &Shader::from_file("shaders/tonemapper/vert.glsl").expect("Could not load voxel vertex shader"),
                &Shader::from_file("shaders/tonemapper/frag.glsl").expect("Could not load voxel fragment shader"),
            ),
            factory,
        }
    }

    pub fn create_hdr_views(factory: &mut gfx_device_gl::Factory, size: (u16, u16))
    -> (
        HdrShaderView,
        HdrRenderView,
        HdrDepthView,
        Sampler<gfx_device_gl::Resources>,
    ) {
        let (_, hdr_shader_view, hdr_render_view) = factory.create_render_target::<HdrFormat>(size.0, size.1).unwrap();
        let hdr_sampler = factory.create_sampler(SamplerInfo::new(FilterMethod::Scale, WrapMode::Clamp));
        let hdr_depth_view = factory.create_depth_stencil_view_only::<HdrDepthFormat>(size.0, size.1).unwrap();
        (hdr_shader_view, hdr_render_view, hdr_depth_view, hdr_sampler)
    }

    pub fn begin_frame(&mut self, clear_color: Option<Vec3<f32>>) {
        if let Some(color) = clear_color {
            self.encoder.clear(&self.color_view, [color.x, color.y, color.z, 1.0]);
            self.encoder.clear(&self.hdr_render_view, [color.x, color.y, color.z]);
        }
        self.encoder.clear_depth(&self.hdr_depth_view, 1.0);
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

    pub fn render_tonemapped_output(&mut self, global_consts: &ConstHandle<GlobalConsts>) {
        let data = tonemapper::PipelineData {
            in_hdr: (self.hdr_shader_view.clone(), self.hdr_sampler.clone()),
            global_consts: global_consts.buffer().clone(),
            out_color: self.color_view.clone(),
        };
        let slice = Slice::<gfx_device_gl::Resources> {
            start: 0,
            end: 3,
            base_vertex: 0,
            instances: None,
            buffer: IndexBuffer::Auto,
        };
        self.encoder.draw(&slice, self.tonemapper_pipeline.pso(), &data);
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

    pub fn hdr_shader_view(&self) -> &HdrShaderView { &self.hdr_shader_view }
    pub fn hdr_render_view(&self) -> &HdrRenderView { &self.hdr_render_view }
    pub fn hdr_depth_view(&self) -> &HdrDepthView { &self.hdr_depth_view }
    pub fn hdr_sampler(&self) -> &Sampler<gfx_device_gl::Resources> { &self.hdr_sampler }

    #[allow(dead_code)]
    pub fn set_views(&mut self, color_view: ColorView, depth_view: DepthView, size: (u16, u16)) {
        let (hdr_shader_view, hdr_render_view, hdr_depth_view, hdr_sampler) = Self::create_hdr_views(&mut self.factory, size);
        self.hdr_shader_view = hdr_shader_view;
        self.hdr_render_view = hdr_render_view;
        self.hdr_depth_view = hdr_depth_view;
        self.hdr_sampler = hdr_sampler;
        self.color_view = color_view;
        self.depth_view = depth_view;
    }
}
