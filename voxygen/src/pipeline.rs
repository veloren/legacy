use gfx::{
    handle::Program,
    pso::{PipelineInit, PipelineState},
    state::{CullFace, FrontFace, MultiSample, RasterMethod, Rasterizer},
    traits::FactoryExt,
    Primitive,
};
use gfx_device_gl;

use crate::shader::Shader;

pub struct Pipeline<P: PipelineInit> {
    #[allow(dead_code)]
    program: Program<gfx_device_gl::Resources>,
    pso: PipelineState<gfx_device_gl::Resources, P::Meta>,
}

impl<P: PipelineInit> Pipeline<P> {
    pub fn new(factory: &mut gfx_device_gl::Factory, pipe: P, vs: &Shader, ps: &Shader) -> Pipeline<P> {
        let program = factory
            .link_program(vs.bytes(), ps.bytes())
            .expect("Failed to compile shader program");
        Pipeline::<P> {
            pso: factory
                .create_pipeline_from_program(
                    &program,
                    Primitive::TriangleList,
                    Rasterizer {
                        front_face: FrontFace::CounterClockwise,
                        cull_face: CullFace::Back,
                        method: RasterMethod::Fill,
                        offset: None,
                        samples: Some(MultiSample),
                    },
                    //Rasterizer::new_fill().with_cull_back(),
                    pipe,
                )
                .expect("Failed to create rendering pipeline"),
            program,
        }
    }

    pub fn pso(&self) -> &PipelineState<gfx_device_gl::Resources, P::Meta> { &self.pso }
}
