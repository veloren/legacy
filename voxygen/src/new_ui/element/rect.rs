// Standard
use std::{
    rc::Rc,
    cell::Cell,
};

// Library
use vek::*;

// Local
use renderer::Renderer;
use super::{
    Element,
    ResCache,
    Span,
};
use super::primitive::draw_rectangle;

#[derive(Clone)]
pub struct Rect {
    col: Cell<Rgba<f32>>,
    padding: Cell<Vec2<Span>>,
}

impl Rect {
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            col: Cell::new(Rgba::one()),
            padding: Cell::new(Span::zero()),
        })
    }

    pub fn with_color(self: Rc<Self>, col: Rgba<f32>) -> Rc<Self> {
        self.col.set(col);
        self
    }

    pub fn with_padding(self: Rc<Self>, padding: Vec2<Span>) -> Rc<Self> {
        self.padding.set(padding);
        self
    }

    pub fn get_color(&self) -> Rgba<f32> { self.col.get() }
    pub fn set_color(&self, col: Rgba<f32>) { self.col.set(col); }

    pub fn get_padding(&self) -> Vec2<Span> { self.padding.get() }
    pub fn set_padding(&self, padding: Vec2<Span>) { self.padding.set(padding); }
}

impl Element for Rect {
    fn deep_clone(&self) -> Rc<dyn Element> {
        Rc::new(self.clone())
    }

    fn render(&self, renderer: &mut Renderer, rescache: &mut ResCache, bounds: (Vec2<f32>, Vec2<f32>)) {
        let res = renderer.get_view_resolution().map(|e| e as f32);
        let padding_rel = self.padding.get().map(|e| e.rel) * bounds.1 + self.padding.get().map(|e| e.px as f32) / res;
        let child_bounds = (bounds.0 + padding_rel, bounds.1 - padding_rel * 2.0);

        draw_rectangle(renderer, rescache, child_bounds.0, child_bounds.1, self.col.get());
    }
}
