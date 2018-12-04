use crate::get_shader_path;
use fnv::FnvBuildHasher;
use gfx::{self, Primitive, Slice};
use gfx_device_gl;
use indexmap::IndexMap;

type FnvIndexMap<K, V> = IndexMap<K, V, FnvBuildHasher>;

use crate::{
    consts::{ConstHandle, GlobalConsts},
    pipeline::Pipeline,
    renderer::{HdrDepthFormat, HdrFormat, Renderer},
    shader::Shader,
    voxel::{mesh::VertexBuffer, MaterialKind, Model, ModelConsts, Vertex},
};

type VoxelPipelineData = voxel_pipeline::Data<gfx_device_gl::Resources>;
type WaterPipelineData = water_pipeline::Data<gfx_device_gl::Resources>;

gfx_defines! {
    pipeline voxel_pipeline {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        model_consts: gfx::ConstantBuffer<ModelConsts> = "model_consts",
        global_consts: gfx::ConstantBuffer<GlobalConsts> = "global_consts",
        out_color: gfx::BlendTarget<HdrFormat> = ("target", gfx::state::ColorMask::all(), gfx::preset::blend::ALPHA),
        out_depth: gfx::DepthTarget<HdrDepthFormat> = gfx::preset::depth::LESS_EQUAL_WRITE,
    }

    pipeline water_pipeline {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        model_consts: gfx::ConstantBuffer<ModelConsts> = "model_consts",
        global_consts: gfx::ConstantBuffer<GlobalConsts> = "global_consts",
        out_color: gfx::BlendTarget<HdrFormat> = ("target", gfx::state::ColorMask::all(), gfx::preset::blend::ALPHA),
        out_depth: gfx::DepthTarget<HdrDepthFormat> = gfx::preset::depth::LESS_EQUAL_WRITE,
    }
}

struct DrawPacket {
    vbuf: VertexBuffer,
    slice: Slice<gfx_device_gl::Resources>,
    model_consts: gfx::handle::Buffer<gfx_device_gl::Resources, ModelConsts>,
    global_consts: gfx::handle::Buffer<gfx_device_gl::Resources, GlobalConsts>,
}

pub struct VolumePipeline {
    voxel_pipeline: Pipeline<voxel_pipeline::Init<'static>>,
    water_pipeline: Pipeline<water_pipeline::Init<'static>>,
    draw_queue: FnvIndexMap<MaterialKind, Vec<DrawPacket>>,
}

impl VolumePipeline {
    pub fn new(renderer: &mut Renderer) -> Self {
        let voxel_pipeline = Pipeline::new(
            renderer.factory_mut(),
            voxel_pipeline::new(),
            &Shader::from_file(get_shader_path("voxel/voxel.vert")).expect("Could not load voxel vertex shader"),
            &Shader::from_file(get_shader_path("voxel/voxel.frag")).expect("Could not load voxel fragment shader"),
        );

        let water_pipeline = Pipeline::new(
            renderer.factory_mut(),
            water_pipeline::new(),
            &Shader::from_file(get_shader_path("voxel/water.vert")).expect("Could not load voxel vertex shader"),
            &Shader::from_file(get_shader_path("voxel/water.frag")).expect("Could not load voxel fragment shader"),
        );

        VolumePipeline {
            voxel_pipeline,
            water_pipeline,
            draw_queue: FnvIndexMap::with_capacity_and_hasher(4, Default::default()),
        }
    }

    pub fn draw_model(
        &mut self,
        model: &Model,
        model_consts: &ConstHandle<ModelConsts>,
        global_consts: &ConstHandle<GlobalConsts>,
    ) {
        model.vbufs().iter().for_each(|(mat, data)| {
            let queued = self.draw_queue.entry(*mat).or_insert(Vec::new());
            let (vbuf, slice) = data;
            // Don't draw models with no vertices TODO: For primitives other TriangleList
            if slice.get_prim_count(Primitive::TriangleList) > 0 {
                queued.push(DrawPacket {
                    vbuf: vbuf.clone(),
                    slice: slice.clone(),
                    model_consts: model_consts.buffer().clone(),
                    global_consts: global_consts.buffer().clone(),
                })
            }
        });
    }

    pub fn flush(&mut self, renderer: &mut Renderer) {
        let out_color = renderer.hdr_render_view().clone();
        let out_depth = renderer.hdr_depth_view().clone();
        let encoder = renderer.encoder_mut();
        let vox_pso = self.voxel_pipeline.pso();
        let water_pso = self.water_pipeline.pso();
        // Sort the draw queue by draw priority. Solid -> Translucent -> Water
        self.draw_queue.sort_keys();
        // Iterate the sorted queue and draw the contained DrawPackets for each kind
        self.draw_queue.iter_mut().for_each(|(mat, ref mut packets)| {
            // Drain the vector of packets so they don't carry over to the next frame
            packets.drain(..).for_each(|packet| match *mat {
                MaterialKind::Water => {
                    let pipe_data = &WaterPipelineData {
                        vbuf: packet.vbuf,
                        model_consts: packet.model_consts,
                        global_consts: packet.global_consts,
                        out_color: out_color.clone(),
                        out_depth: out_depth.clone(),
                    };
                    encoder.draw(&packet.slice, water_pso, pipe_data);
                },
                _ => {
                    let pipe_data = &VoxelPipelineData {
                        vbuf: packet.vbuf,
                        model_consts: packet.model_consts,
                        global_consts: packet.global_consts,
                        out_color: out_color.clone(),
                        out_depth: out_depth.clone(),
                    };
                    encoder.draw(&packet.slice, vox_pso, pipe_data);
                },
            });
        });
    }
}
