// Standard
use std::{cell::Cell, hash::Hash};

// Library
use vek::*;

// Library
use gfx::traits::FactoryExt;
use gfx_glyph::{GlyphBrush, GlyphBrushBuilder, Scale, Section};
use lyon::{
    math::rect,
    tessellation::{
        basic_shapes::fill_rectangle,
        geometry_builder::{BuffersBuilder, VertexBuffers},
        FillOptions,
    },
};

// Local
use super::{
    render::{create_fill_pso, fill_pipeline, FillVertex, VertexFactory},
    rescache::{GlyphBrushRes, RectVboRes, ResCache},
};
use renderer::Renderer;
use shader::Shader;

fn create_rect_vbo(renderer: &mut Renderer, pos: Vec2<f32>, sz: Vec2<f32>, col: Rgba<f32>) -> RectVboRes {
    let mut mesh: VertexBuffers<FillVertex, u16> = VertexBuffers::new();

    fill_rectangle(
        &rect(pos.x, pos.y, sz.x, sz.y),
        &FillOptions::tolerance(0.0001),
        &mut BuffersBuilder::new(&mut mesh, VertexFactory::with_color(col)),
    );

    renderer
        .factory_mut()
        .create_vertex_buffer_with_slice(&mesh.vertices[..], &mesh.indices[..])
}

pub(crate) fn draw_rectangle(
    renderer: &mut Renderer,
    rescache: &mut ResCache,
    pos: Vec2<f32>,
    sz: Vec2<f32>,
    col: Rgba<f32>,
) {
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

// TODO: Don't hard-code this
static UI_FONT: &[u8] = include_bytes!("../../../fonts/fantasque-sans-mono-regular.ttf");

fn create_glyph_brush(renderer: &mut Renderer, font: &'static [u8]) -> GlyphBrushRes {
    GlyphBrushBuilder::using_font_bytes(font).build(renderer.factory().clone())
}

pub(crate) fn draw_text(
    renderer: &mut Renderer,
    rescache: &mut ResCache,
    text: &str,
    pos: Vec2<f32>,
    sz: Vec2<f32>,
    col: Rgba<f32>,
) {
    // TODO: Properly hash all unique details of this glyph brush
    let brush = rescache.get_or_create_glyph_brush(0, || create_glyph_brush(renderer, UI_FONT));

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

    brush
        .borrow_mut()
        .draw_queued(renderer.encoder_mut(), &color_view, &depth_view);
}
