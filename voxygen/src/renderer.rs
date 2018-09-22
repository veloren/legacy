// Library
use vek::*;
use gfx::{
    self,
    format::Formatted,
    handle::{DepthStencilView, RenderTargetView, Sampler, ShaderResourceView},
    texture::{FilterMethod, SamplerInfo, WrapMode},
    Device, Encoder, Factory,
};
use gfx_device_gl;

pub type HdrFormat = (gfx::format::R16_G16_B16_A16, gfx::format::Float);
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
}

impl Renderer {
    pub fn new(
        device: gfx_device_gl::Device,
        mut factory: gfx_device_gl::Factory,
        color_view: ColorView,
        depth_view: DepthView,
        size: (u16, u16),
    ) -> Renderer {
        let (hdr_shader_view, hdr_render_view, hdr_depth_view, hdr_sampler) =
            Self::create_hdr_views(&mut factory, size);
        Renderer {
            device,
            color_view,
            depth_view,
            hdr_shader_view,
            hdr_render_view,
            hdr_depth_view,
            hdr_sampler,
            encoder: factory.create_command_buffer().into(),
            factory,
        }
    }

    pub fn create_hdr_views(
        factory: &mut gfx_device_gl::Factory,
        size: (u16, u16),
    ) -> (
        HdrShaderView,
        HdrRenderView,
        HdrDepthView,
        Sampler<gfx_device_gl::Resources>,
    ) {
        let (_, hdr_shader_view, hdr_render_view) = factory.create_render_target::<HdrFormat>(size.0, size.1).unwrap();
        let hdr_sampler = factory.create_sampler(SamplerInfo::new(FilterMethod::Scale, WrapMode::Clamp));
        let hdr_depth_view = factory
            .create_depth_stencil_view_only::<HdrDepthFormat>(size.0, size.1)
            .unwrap();
        (hdr_shader_view, hdr_render_view, hdr_depth_view, hdr_sampler)
    }

    pub fn begin_frame(&mut self, clear_color: Option<Vec3<f32>>) {
        if let Some(color) = clear_color {
            self.encoder.clear(&self.color_view, [color.x, color.y, color.z, 1.0]);
            self.encoder
                .clear(&self.hdr_render_view, [color.x, color.y, color.z, 1.0]);
        }
        self.encoder.clear_depth(&self.hdr_depth_view, 1.0);
    }

    pub fn end_frame(&mut self) {
        self.encoder.flush(&mut self.device);
        self.device.cleanup();
    }

    #[allow(dead_code)]
    pub fn encoder(&self) -> &Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer> {
        &self.encoder
    }
    #[allow(dead_code)]
    pub fn encoder_mut(&mut self) -> &mut Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer> {
        &mut self.encoder
    }

    #[allow(dead_code)]
    pub fn factory(&self) -> &gfx_device_gl::Factory { &self.factory }
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

    pub fn get_view_resolution(&self) -> Vec2<u16> {
        Vec2::new(
            self.color_view.get_dimensions().0,
            self.color_view.get_dimensions().1,
        )
    }

    #[allow(dead_code)]
    pub fn set_views(&mut self, color_view: ColorView, depth_view: DepthView, size: (u16, u16)) {
        let (hdr_shader_view, hdr_render_view, hdr_depth_view, hdr_sampler) =
            Self::create_hdr_views(&mut self.factory, size);
        self.hdr_shader_view = hdr_shader_view;
        self.hdr_render_view = hdr_render_view;
        self.hdr_depth_view = hdr_depth_view;
        self.hdr_sampler = hdr_sampler;
        self.color_view = color_view;
        self.depth_view = depth_view;
    }
}
