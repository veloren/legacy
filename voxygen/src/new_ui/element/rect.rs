// Standard
use std::{cell::Cell, rc::Rc};

// Library
use vek::*;

// Local
use super::{primitive::draw_rectangle, Element, ResCache, Span, Bounds};
use renderer::Renderer;

#[allow(dead_code)]
#[derive(Clone)]
pub struct Rect {
    col: Cell<Rgba<f32>>,
    padding: Cell<Vec2<Span>>,
}

impl Rect {
    #[allow(dead_code)]
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            col: Cell::new(Rgba::one()),
            padding: Cell::new(Span::zero()),
        })
    }

    #[allow(dead_code)]
    pub fn with_color(self: Rc<Self>, col: Rgba<f32>) -> Rc<Self> {
        self.col.set(col);
        self
    }

    #[allow(dead_code)]
    pub fn with_padding(self: Rc<Self>, padding: Vec2<Span>) -> Rc<Self> {
        self.padding.set(padding);
        self
    }

    #[allow(dead_code)]
    pub fn get_color(&self) -> Rgba<f32> { self.col.get() }
    #[allow(dead_code)]
    pub fn set_color(&self, col: Rgba<f32>) { self.col.set(col); }

    #[allow(dead_code)]
    pub fn get_padding(&self) -> Vec2<Span> { self.padding.get() }
    #[allow(dead_code)]
    pub fn set_padding(&self, padding: Vec2<Span>) { self.padding.set(padding); }

    #[allow(dead_code)]
    pub fn clone_all(&self) -> Rc<Self> { Rc::new(self.clone()) }
}

impl Element for Rect {
    fn deep_clone(&self) -> Rc<dyn Element> { self.clone_all() }

    fn render(&self, renderer: &mut Renderer, rescache: &mut ResCache, bounds: Bounds) {
        let res = renderer.get_view_resolution().map(|e| e as f32);
        let padding_rel = self.padding.get().map(|e| e.rel) * bounds.1 + self.padding.get().map(|e| e.px as f32) / res;
        let child_bounds = (bounds.0 + padding_rel, bounds.1 - padding_rel * 2.0);

        draw_rectangle(renderer, rescache, child_bounds.0, child_bounds.1, self.col.get());
    }
}
