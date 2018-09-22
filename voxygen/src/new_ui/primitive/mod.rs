// Standard
use std::hash::Hash;

// Library
use vek::*;

// Library
use lyon::{
    math::rect,
    tessellation::{
        FillOptions,
        basic_shapes::fill_rectangle,
        geometry_builder::{BuffersBuilder, VertexBuffers},
    },
};
use gfx::traits::FactoryExt;
use gfx_glyph::{GlyphBrush, GlyphBrushBuilder, Section, Scale};

// Local
use renderer::Renderer;
use shader::Shader;
use super::{
    rescache::{RectVboRes, GlyphBrushRes, ResCache},
    render::{create_fill_pso, fill_pipeline, FillVertex, VertexFactory},
};

fn create_rect_vbo(renderer: &mut Renderer, pos: Vec2<f32>, sz: Vec2<f32>, col: Rgba<f32>) -> RectVboRes {
    let mut mesh: VertexBuffers<FillVertex, u16> = VertexBuffers::new();

    fill_rectangle(
        &rect(pos.x, pos.y, sz.x, sz.y),
        &FillOptions::tolerance(0.0001),
        &mut BuffersBuilder::new(&mut mesh, VertexFactory::with_color(col)),
    );

    renderer.factory_mut().create_vertex_buffer_with_slice(&mesh.vertices[..], &mesh.indices[..])
}

// TODO: Don't hard-code this
static UI_FONT: &[u8] = include_bytes!("../../../assets/voxygen/fonts/NotoSans-Regular.ttf");

fn create_glyph_brush(renderer: &mut Renderer, text: &str, pos: Vec2<f32>, sz: Vec2<f32>, col: Rgba<f32>) -> GlyphBrushRes {
    GlyphBrushBuilder::using_font_bytes(UI_FONT).build(renderer.factory().clone())
}

pub(crate) fn draw_rectangle(renderer: &mut Renderer, rescache: &mut ResCache, pos: Vec2<f32>, sz: Vec2<f32>, col: Rgba<f32>) {
    let pso = rescache.get_or_create_fill_pso(|| create_fill_pso(renderer));
    let rect_vbo = rescache.get_or_create_rect_vbo(pos, sz, col, || create_rect_vbo(renderer, pos, sz, col));

    let color_view = renderer.color_view().clone();

    renderer.encoder_mut().draw(
        &rect_vbo.1,
        &pso,
        &fill_pipeline::Data {
            vbo: rect_vbo.0.clone(),
            out_color: color_view,
        },
    );
}

pub(crate) fn draw_text(renderer: &mut Renderer, rescache: &mut ResCache, text: &str, pos: Vec2<f32>, sz: Vec2<f32>, col: Rgba<f32>) {
    let brush = rescache.get_or_create_glyph_brush(text, pos, sz, col, || create_glyph_brush(renderer, text, pos, sz, col));

    let color_view = renderer.color_view().clone();
    let depth_view = renderer.depth_view().clone();

    let res = renderer.get_view_resolution().map(|e| e as f32);

    brush.borrow_mut().queue(Section {
        text,
        screen_position: (pos * res).into_tuple(),
        scale: Scale { x: sz.x, y: sz.y },
        color: col.into_array(),
        ..Section::default()
    });

    brush.borrow_mut().draw_queued(renderer.encoder_mut(), &color_view, &depth_view);
}
