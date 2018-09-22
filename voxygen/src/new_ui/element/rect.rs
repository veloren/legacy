// Standard
use std::{
    rc::Rc,
    cell::RefCell
};

// Library
use vek::*;

// Local
use renderer::Renderer;
use super::{
    Element,
    ResCache,
    Pos,
    Size,
};
use super::primitive::draw_rectangle;

#[derive(Clone)]
pub struct Rect {
    col: Rgba<f32>,
    padding: Size,
}

impl Rect {
    pub fn new() -> Self {
        Self {
            col: Rgba::zero(),
            padding: Size::zero(),
        }
    }

    pub fn with_color(mut self, col: Rgba<f32>) -> Self {
        self.col = col;
        self
    }

    pub fn with_padding(mut self, padding: Size) -> Self {
        self.padding = padding;
        self
    }
}

impl Element for Rect {
    fn deep_clone(&self) -> Rc<RefCell<dyn Element>> {
        Rc::new(RefCell::new(self.clone()))
    }

    fn render(&self, renderer: &mut Renderer, rescache: &mut ResCache, bounds: (Vec2<f32>, Vec2<f32>)) {
        let res = renderer.get_view_resolution().map(|e| e as f32);
        let padding_rel = self.padding.rel * bounds.1 + self.padding.px.map(|e| e as f32) / res;
        let child_bounds = (bounds.0 + padding_rel, bounds.1 - padding_rel * 2.0);

        draw_rectangle(renderer, rescache, child_bounds.0, child_bounds.1, self.col);
    }
}
